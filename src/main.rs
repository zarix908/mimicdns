mod result;

use crate::result::Err;
use base64::{Engine, engine::general_purpose::STANDARD};
use domain::{
    base::{MessageBuilder, Name, Rtype},
    rdata::Txt,
};
use result::Result;
use std::{net::UdpSocket, str::FromStr};

const MTU: u16 = 1600;

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

    println!("wrapped message successfully");
}

fn run() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5555").map_err(Err::Bind)?;

    println!("listen...");

    let mut buf = [0u8; MTU as usize];
    let (count, addr) = socket.recv_from(&mut buf).map_err(Err::Receive)?;
    let data = &buf[0..count];

    let mut data_base64 = [0u8; (MTU * 4 / 3 + 4) as usize];
    let count = STANDARD
        .encode_slice(data, &mut data_base64)
        .map_err(Err::Base64Encode)?;
    let data = &data_base64[0..count];

    let mut msg = MessageBuilder::new_vec();
    
    msg.header_mut().set_qr(true);

    let mut msg = msg.question();
    let name = Name::<Vec<u8>>::from_str("txt.zarix908.com").unwrap();
    msg.push((&name, Rtype::TXT))
        .map_err(Err::BuildDnsQuestion)?;

    let mut msg = msg.answer();
    let txt = Txt::<Vec<u8>>::build_from_slice(data).map_err(Err::BuildTxtRecord)?;
    msg.push((&name, 60, txt)).map_err(Err::BuildDnsAnswer)?;

    let target = msg.finish();

    socket.send_to(&target, addr).map_err(Err::Send)?;

    Ok(())
}
