use bytes::BytesMut;
use crate::header::Header;
use crate::question::Question;
#[allow(unused)]
pub struct Message {
    header: Header,
    question: Question,
}

impl Message {
    pub fn for_second_test() -> BytesMut {
        let mut header = Header::default();
        let question = Question::for_question_test();
        header.increment_qd_count();
        let (mut header, question) = (header.serialize().unwrap(), question.serialize());
        header.extend_from_slice(&question);
        header
    }
}