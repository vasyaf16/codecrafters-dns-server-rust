
use std::ops::{Deref, DerefMut};
use anyhow::bail;
use bytes::{BufMut, BytesMut};
use nom::AsBytes;
use crate::answer::Answer;
use crate::header::Header;
use crate::question::Question;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Message {
    header: Header,
    question: Question,
    answer: Answer,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Class {
    IN = 1,
    CS = 2,
    CH = 3,
    HS = 4,
}

impl TryFrom<u16> for Class {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::IN),
            2 => Ok(Self::CS),
            3 => Ok(Self::CH),
            4 => Ok(Self::HS),
            _ => bail!("can construct enum variants only from u16 from 1 to 4")
        }
    }
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Ty {
    A = 1,
    NS = 2,
    CNAME = 5,
    SOA = 6,
    WKS = 11,
    PTR = 12,
    HINFO = 13,
    MINFO = 14,
    MX = 15,
    TXT = 16,
}

impl TryFrom<u16> for Ty {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::A),
            2 => Ok(Self::NS),
            5 => Ok(Self::CNAME),
            6 => Ok(Self::SOA),
            11 => Ok(Self::WKS),
            12 => Ok(Self::PTR),
            13 => Ok(Self::HINFO),
            14 => Ok(Self::MINFO),
            15 => Ok(Self::MX),
            16 => Ok(Self::TXT),
            _ => bail!("unexpected value")
        }
    }
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Label {
    val: BytesMut,
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Labels(Vec<Label>);
impl Deref for Labels {
    type Target = Vec<Label>;

    fn deref(&self) -> &Self::Target {
        & self.0
    }
}

impl DerefMut for Labels {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Labels {
    pub fn from_bytes(seq: &[u8]) -> Self {
        let mut iter = seq.into_iter().copied();
        let mut vec = vec![];
        while let Some(len) = iter.next() {
            let label = Label::from(iter.by_ref().take(len as usize).collect::<Vec<_>>().as_bytes());
            vec.push(label)
        };
        Self(vec)
    }

    pub fn from_domain(domain: &str) -> Self {
        Self(domain.split(".").map(|s| Label::from(s.as_bytes())).collect::<Vec<_>>())
    }

    pub fn into_bytes_mut(self) -> BytesMut {
        self.0.iter().flat_map(|label| label.as_bytes()).collect()
    }
}


impl<'a> From<&'a [u8]> for Label {
    fn from(value: &'a [u8]) -> Self {
        let mut val = BytesMut::new();
        let len = value.len() as u8;
        val.put_u8(len);
        val.extend_from_slice(value);
        Self { val }
    }
}

impl Deref for Label {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl DerefMut for Label {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl Message {

    pub fn new(header: Header, question: Question, answer: Answer) -> Self {
        Self {
            header,
            question,
            answer,
        }
    }

    pub fn serialize(self) -> BytesMut {
        let (mut header, question, answer) = (
            self.header.serialize().unwrap(),
            self.question.serialize(),
            self.answer.serialize()
        );
        header.extend_from_slice(&question);
        header.extend_from_slice(&answer);
        header
    }
    pub fn produce_full_default_message() -> BytesMut {
        let mut header = Header::default();
        let question = Question::for_question_test();
        header.increment_qd_count();
        let answer = Answer::for_third_test();
        header.increment_an_count();
        let message = Self::new(header,question,answer);
        message.serialize()
    }
}

#[cfg(test)]
mod tests{
    use bytes::BytesMut;
    use crate::message::{Label, Labels};

    #[test]
    fn label_test() {
        let v = Labels::from_bytes(b"\x0ccodecrafters\x02io");
        println!("{:?}", v);
    }

    #[test]
    fn test_from_domain() {
        let domain = "codecrafters.io";
        let expected = Labels(vec![
            Label{val: BytesMut::from("\x0ccodecrafters")},
            Label{val: BytesMut::from("\x02io")}
        ]);
        let x = Labels::from_domain(domain);
        assert_eq!(x, expected)
    }
}