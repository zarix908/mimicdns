use std::str::FromStr;

use crate::{
    ChunkStore, constant, result::{Err, Result}
};
use domain::{
    base::{self, Message, RecordSection, iana, message, message_builder},
    dep::octseq::{self, Array},
    rdata,
};

pub struct DnsWrapper {
    domain: base::Name<octseq::Array<18>>,
    question: message::Message<Vec<u8>>,
}

impl DnsWrapper {
    pub fn new() -> DnsWrapper {
        let domain = base::Name::<octseq::Array<18>>::from_str("txt.zarix908.com").unwrap();
        let question = Self::build_question(&domain).unwrap();

        DnsWrapper { domain, question }
    }

    pub fn wrap_to(&self, question_data: &mut [u8], target: &mut Array<{ constant::MTU }>, chunk_store: ChunkStore) -> Result<()> {
        let question_msg = Message::from_slice(question_data).unwrap();
        let question = question_msg.question().nth(0).unwrap().unwrap();

        let mut msg = message_builder::MessageBuilder::from_target(target)
            .unwrap()
            .start_answer(question_msg, iana::Rcode::NOERROR)
            .map_err(Err::BuildDnsQuestion)?;

        let chunk_store = chunk_store.lock().unwrap();
        let data = chunk_store.get(&question_msg.header().id()).unwrap();

        let txt = rdata::Txt::<Array<{ constant::MAX_PAYLOAD_LEN }>>::build_from_slice(data)
            .map_err(Err::BuildTxtRecord)?;
        msg.push((&question.qname(), 60, txt))
            .map_err(Err::BuildDnsAnswer)?;

        msg.finish();
        Ok(())
    }

    pub fn unwrap(data: &[u8]) -> RecordSection<'_, [u8]> {
        let msg = base::Message::from_slice(data).unwrap();
        msg.answer().unwrap()
    }

    fn build_question(domain: &base::Name<octseq::Array<18>>) -> Result<base::Message<Vec<u8>>> {
        let mut msg = base::MessageBuilder::new_vec();
        msg.header_mut().set_qr(true);

        let mut msg = msg.question();
        msg.push((domain, iana::Rtype::TXT))
            .map_err(Err::BuildDnsQuestion)?;

        Ok(base::Message::from_octets(msg.finish()).unwrap())
    }
}
