mod result;

use crate::result::Err;
use base64::{Engine, engine::general_purpose::STANDARD};
use domain::{
    base::{Message, MessageBuilder, Name, Rtype, iana::Rcode}, dep::octseq::Array, rdata::Txt
};
use result::Result;
use std::{net::UdpSocket, str::FromStr};

const DNS_WRAP_LEN: usize = 100;
const MTU: usize = 1600;
const INPUT_MTU: usize = MTU - DNS_WRAP_LEN;

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
    
    let mut data = [0u8; INPUT_MTU];
    let mut data_base64 = [0u8; INPUT_MTU * 4 / 3 + 4];

    let mut target = Vec::new();
    let name = Name::<Array<18>>::from_str("txt.zarix908.com").unwrap();
    let question = build_question(&name)?;
    let msg = Message::from_slice(&question).unwrap(); 

    println!("listen...");

    loop {
        let (count, addr) = recv_socket.recv_from(&mut data).map_err(Err::Receive)?;
        let data = &data[0..count];

        let count = STANDARD
            .encode_slice(data, &mut data_base64)
            .map_err(Err::Base64Encode)?;
        let data = &data_base64[0..count];

        let answer = build_answer(target, msg, &name, data)?;
        send_socket.send_to(&answer, addr).map_err(Err::Send)?;
        target = answer
    }
}

fn build_question(name: &Name<Array<18>>) -> Result<Vec<u8>> {
    let mut msg = MessageBuilder::new_vec();
    msg.header_mut().set_qr(true);

    let mut msg = msg.question();
    msg.push((name, Rtype::TXT)).map_err(Err::BuildDnsQuestion)?;

    Ok(msg.finish())
}

fn build_answer(
    target: Vec<u8>,
    question_msg: &Message<[u8]>, 
    name: &Name<Array<18>>, 
    data: &[u8],
) -> Result<Vec<u8>> {
    let mut msg = MessageBuilder::from_target(target).unwrap().start_answer(question_msg, Rcode::NOERROR).map_err(Err::BuildDnsQuestion)?;

    let txt = Txt::<Vec<u8>>::build_from_slice(data).map_err(Err::BuildTxtRecord)?;
    msg.push((name, 60, txt)).map_err(Err::BuildDnsAnswer)?;

    Ok(msg.finish())
}
