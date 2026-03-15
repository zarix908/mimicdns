use std::str::FromStr;

use domain::{base::{self, iana, message, message_builder}, dep::octseq::{self}, rdata};
use crate::{result::{Err, Result}};

pub struct DnsWrapper {
    domain: base::Name::<octseq::Array<18>>,
    question: message::Message<Vec<u8>>,
}

impl DnsWrapper {
    pub fn new() -> DnsWrapper {
        let domain = base::Name::<octseq::Array<18>>::from_str("txt.zarix908.com").unwrap();
        let question = Self::build_question(&domain).unwrap();

        DnsWrapper{domain, question}
    }

    pub fn wrap_to(&self, target: &mut Vec<u8>, data: &[u8]) -> Result<()> {
        let mut msg = message_builder::MessageBuilder::from_target(target).unwrap().start_answer(&self.question, iana::Rcode::NOERROR).map_err(Err::BuildDnsQuestion)?;

        let txt = rdata::Txt::<Vec<u8>>::build_from_slice(data).map_err(Err::BuildTxtRecord)?;
        msg.push((&self.domain, 60, txt)).map_err(Err::BuildDnsAnswer)?;

        msg.finish();
        Ok(())
    }

    fn build_question(domain: &base::Name<octseq::Array<18>>) -> Result<base::Message<Vec<u8>>> {
        let mut msg = base::MessageBuilder::new_vec();
        msg.header_mut().set_qr(true);

        let mut msg = msg.question();
        msg.push((domain, iana::Rtype::TXT)).map_err(Err::BuildDnsQuestion)?;

        Ok(base::Message::from_octets(msg.finish()).unwrap())
    }
}
