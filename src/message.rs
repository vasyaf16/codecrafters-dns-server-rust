use std::io::Read;
use std::ops::{Deref, DerefMut};
use anyhow::bail;
use bytes::{BufMut, BytesMut};
use nom::AsBytes;
use crate::answer::Answer;
use crate::header::Header;
use crate::question::Question;

#[allow(unused)]
pub struct Message {
    header: Header,
    question: Question,
    answer: Answer,
}

#[repr(u16)]
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
impl Label {
    pub fn from_sequence(seq: &[u8]) -> Vec<Self> {
        let mut iter = seq.into_iter().copied();
        let mut vec = vec![];
        while let Some(len) = iter.next() {
            let label = Label::from(iter.by_ref().take(len as usize).collect::<Vec<_>>().as_bytes());
            vec.push(label)
        };
        vec
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
    pub fn for_second_test() -> BytesMut {
        let mut header = Header::default();
        let question = Question::for_question_test();
        header.increment_qd_count();
        let (mut header, question) = (header.serialize().unwrap(), question.serialize());
        header.extend_from_slice(&question);
        header
    }
}

#[cfg(test)]
mod tests{
    use crate::message::Label;

    #[test]
    fn label_test() {
        let v = Label::from_sequence(b"\x0ccodecrafters\x02io");
        println!("{:?}", v);
    }
}