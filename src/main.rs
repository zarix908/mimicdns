mod constant;
mod dnswrap;
mod result;

use crate::dnswrap::DnsWrapper;
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::BytesMut;
use crossbeam_channel::{Receiver, Sender};
use domain::{base::Message, dep::octseq::Array};
use result::{Err, Result};
use std::{collections::HashMap, net::UdpSocket, process::exit, sync::{Arc, Mutex}, thread};

type ChunkStore = Arc<Mutex<HashMap<u16, BytesMut>>>;

fn main() {
    thread::scope(|s| {
        // s.spawn(|| {
        //     start_unwrap().unwrap_or_else(handle_err);
        // });

        let chunk_store: ChunkStore = Arc::new(Mutex::new(HashMap::new()));
        let (free_chunk_tx, free_chunk_rx) = crossbeam_channel::bounded::<BytesMut>(100);

        s.spawn(|| {
            run_data_srv(chunk_store, free_chunk_rx).unwrap_or_else(handle_err);
        });
    });
}

fn handle_err(e: Err) {
    match e {
        Err::Bind(e) => println!("failed to bind socket: {}", e),
        Err::Receive(e) => println!("failed to receive messgae: {}", e),
        Err::Base64Encode(e) => println!("failed to encode message: {}", e),
        Err::Base64Decode(e) => println!("failed to decode base64: {}", e),
        Err::BuildDnsQuestion(e) => println!("failed to build dns question: {}", e),
        Err::BuildTxtRecord(e) => println!("failed to build txt record: {}", e),
        Err::BuildDnsAnswer(e) => println!("failed to build dns answer: {}", e),
        Err::Send(e) => println!("failed to send dns message: {}", e),
        Err::Lock(e) => println!("failed to acquire lock: {}", e),
        Err::FreePacketRecv(e) => println!("failed to recv free packet: {}", e),
    }

    exit(1);
}

fn run_data_srv(chunk_store: ChunkStore, free_chunks: Receiver<BytesMut>) -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5555").map_err(Err::Bind)?;

    let mut data = [0u8; constant::MAX_DATA_LEN];

    println!("listen...");

    loop {
        let (count, _) = socket.recv_from(&mut data).map_err(Err::Receive)?;
        let data = &data[0..count];

        let mut chunk = free_chunks.recv().map_err(Err::FreePacketRecv)?;

        let count = STANDARD
            .encode_slice(data, &mut chunk)
            .map_err(Err::Base64Encode)?;
        chunk.truncate(count);

        let mut chunk_store = chunk_store.lock().map_err(|e| Err::Lock(format!("{e}")))?;
        chunk_store.insert(125, chunk);
    }
}

fn run_dns_srv(chunk_store: ChunkStore, free_chunks: Sender<BytesMut>) -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5354").map_err(Err::Bind)?;
    let mut data = [0u8; constant::MTU];

    let dns_wrapper = DnsWrapper::new();
    let mut dns_packet = Array::<{ constant::MTU }>::new();

    loop {
        socket.recv_from(&mut data).map_err(Err::Receive)?;
        dns_wrapper.wrap_to(&mut dns_packet, data);
    }
}

fn start_unwrap() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5556").map_err(Err::Bind)?;
    let mut buf = [0u8; constant::MTU];
    let mut data_base64 = [0u8; constant::MAX_PAYLOAD_LEN];

    loop {
        let count = socket.recv(&mut buf).map_err(Err::Receive)?;
        let records = DnsWrapper::unwrap(&buf[..count]);

        for record in records.flatten() {
            if let Ok(Some(record)) = record.into_record::<domain::rdata::Txt<&[u8]>>() {
                let data: Vec<u8> = record.data().iter().flatten().copied().collect();
                let count = STANDARD
                    .decode_slice(data, &mut data_base64)
                    .map_err(Err::Base64Decode)?;
                let data = &data_base64[..count];

                println!("count: {}", count);

                println!("data: {}", str::from_utf8(data).unwrap());
            }
        }
    }
}
