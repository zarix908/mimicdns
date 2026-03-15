mod result;
mod dnswrap;

use crate::{dnswrap::DnsWrapper, result::Err};
use base64::{Engine, engine::general_purpose::STANDARD};
use result::Result;
use std::net::UdpSocket;

const DNS_WRAP_LEN: usize = 110;
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

    let mut dns_packet = Vec::new();
    let dns_wrapper = DnsWrapper::new();

    println!("listen...");

    loop {
        let (count, addr) = recv_socket.recv_from(&mut data).map_err(Err::Receive)?;
        let data = &data[0..count];

        let count = STANDARD
            .encode_slice(data, &mut data_base64)
            .map_err(Err::Base64Encode)?;
        let data = &data_base64[0..count];

        dns_wrapper.wrap_to(&mut dns_packet, data)?;
        send_socket.send_to(&dns_packet, addr).map_err(Err::Send)?;
    }
}
