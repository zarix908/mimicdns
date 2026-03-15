use std::io;
use std::result;

use base64::EncodeSliceError;
use domain::base::message_builder::PushError;
use domain::rdata::rfc1035::TxtAppendError;

pub type Result<T> = result::Result<T, Err>;

#[derive(Debug)]
pub enum Err {
    Bind(io::Error),
    Receive(io::Error),
    Base64Encode(EncodeSliceError),
    BuildTxtRecord(TxtAppendError),
    BuildDnsQuestion(PushError),
    BuildDnsAnswer(PushError),
    Send(io::Error),
}
