mod dnswrap;
mod result;

use crate::dnswrap::DnsWrapper;
use base64::{Engine, engine::general_purpose::STANDARD};
use result::{Err, Result};
use std::{net::UdpSocket, thread};

const DNS_WRAP_LEN: usize = 110;
const MTU: usize = 1600;
const INPUT_MTU: usize = MTU - DNS_WRAP_LEN;

fn main() {
    thread::scope(|s| {
        s.spawn(|| {
            start_unwrap().unwrap_or_else(handle_err);
        });

        s.spawn(|| {
            start_wrap().unwrap_or_else(handle_err);
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
    }
}

fn start_wrap() -> Result<()> {
    let recv_socket = UdpSocket::bind("127.0.0.1:5555").map_err(Err::Bind)?;
    let send_socket = UdpSocket::bind(("127.0.0.1", 30006)).map_err(Err::Send)?;

    let mut data = [0u8; INPUT_MTU];
    let mut data_base64 = [0u8; INPUT_MTU * 4 / 3 + 4];

    let mut dns_packet = Vec::new();
    let dns_wrapper = DnsWrapper::new();

    println!("listen...");

    loop {
        let (count, _) = recv_socket.recv_from(&mut data).map_err(Err::Receive)?;
        let data = &data[0..count];

        let count = STANDARD
            .encode_slice(data, &mut data_base64)
            .map_err(Err::Base64Encode)?;
        let data = &data_base64[0..count];

        dns_wrapper.wrap_to(&mut dns_packet, data)?;
        send_socket
            .send_to(&dns_packet, ("127.0.0.1", 5556))
            .map_err(Err::Send)?;
    }
}

fn start_unwrap() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5556").map_err(Err::Bind)?;
    let mut buf = [0u8; MTU];
    let mut data_base64 = [0u8; MTU * 4 / 3 + 4];

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

                println!("data: {}", str::from_utf8(data).unwrap());
            }
        }
    }
}
