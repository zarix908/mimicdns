mod result;

use crate::result::Err;
use base64::{Engine, engine::general_purpose::STANDARD};
use domain::{
    base::{MessageBuilder, Name, Rtype}, dep::octseq::Array, rdata::Txt
};
use result::Result;
use std::{net::{SocketAddr, UdpSocket}, str::FromStr};

const MTU: usize = 1600;

fn main() {
    if let Err(e) = run() {
        match e {
            Err::Bind(e) => println!("failed to bind socket: {}", e),
            Err::Receive(e) => println!("failed to receive messgae: {}", e),
            Err::Base64Encode(e) => println!("failed to encode message: {}", e),
            Err::BuildDnsQuestion(e) => println!("failed to build dns question: {}", e),
            Err::BuildTxtRecord(e) => println!("failed to build txt record: {}", e),
            Err::BuildDnsAnswer(e) => println!("failed to build dns answer: {}", e),
            Err::Send(e) => println!("failed to send dns message: {}", e),
        }

        return;
    }
}

fn run() -> Result<()> {
    let recv_socket = UdpSocket::bind("127.0.0.1:5555").map_err(Err::Bind)?;
    let send_socket = UdpSocket::bind(("127.0.0.1", 30006)).map_err(Err::Send)?;
    
    let mut data = [0u8; MTU];
    let mut data_base64 = [0u8; MTU * 4 / 3 + 4];

    let mut target: Option<Vec<u8>> = None;

    println!("listen...");

    let name = Name::<Array<MTU>>::from_str("txt.zarix908.com").unwrap();

    loop {
        let (count, addr) = recv_socket.recv_from(&mut data).map_err(Err::Receive)?;
        let data = &data[0..count];

        let count = STANDARD
            .encode_slice(data, &mut data_base64)
            .map_err(Err::Base64Encode)?;
        let data = &data_base64[0..count];

        let buf = target.take().unwrap_or_else(Vec::new);
        send_answer(buf, &name, data, &send_socket, addr)?;
    }
}

fn send_answer(
    target: Vec<u8>, 
    name: &Name<Array<MTU>>, 
    data: &[u8],
    socket: &UdpSocket, 
    addr: SocketAddr,
) -> Result<Vec<u8>> {
    let mut msg = MessageBuilder::from_target(target).unwrap();
    msg.header_mut().set_qr(true);

    let mut msg = msg.question();
    msg.push((name, Rtype::TXT)).map_err(Err::BuildDnsQuestion)?;

    let mut msg = msg.answer();
    let txt = Txt::<Vec<u8>>::build_from_slice(data).map_err(Err::BuildTxtRecord)?;
    msg.push((name, 60, txt)).map_err(Err::BuildDnsAnswer)?;

    let data = msg.finish();
    socket.send_to(&data, addr).map_err(Err::Send)?;

    println!("sent");

    Ok(data)
}
