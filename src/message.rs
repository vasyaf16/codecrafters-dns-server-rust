
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
    pub(crate) questions: Questions,
    pub(crate) answers: Answers,
}

impl Default for Message {
    fn default() -> Self {
        let header = Header::default();
        Self {header,
            questions: Default::default(),
            answers: Default::default()}
    }
}

impl Message {
    #[allow(unused)]
    pub fn new(header: Header, questions: Vec<Question>, answers: Vec<Answer>) -> Self {
        Self {
            header,
            questions: Questions(questions),
            answers: Answers(answers),
        }
    }

    pub fn serialize(self) -> BytesMut {
        let (mut header, question, answer) = (
            self.header.serialize().unwrap(),
            self.questions.serialize(),
            self.answers.serialize()
        );
        header.extend_from_slice(&question);
        header.extend_from_slice(&answer);
        header
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let header = Header::deserialize(&bytes[..12]).unwrap();
        let (qn, an) = (header.qd(), header.an());
        let mut start = 12usize;
        let mut question = Questions::default();
        // println!("{:?}", header);
        for _ in 0..qn {
            let (q, end) = Question::deserialize(bytes, start);
            // println!("{:#?} \n end is {end}", q);
            start = end;
            question.push(q);
        }
        let mut answer = Answers::default();
        for _ in 0..an {
            let (a, end) = Answer::deserialize(bytes, start);
            start += end;
            answer.push(a);
        }

        Self {
            header,
            questions: question,
            answers: answer,
        }
    }

    pub fn split(self) -> Vec<Self> {
        self.questions.into_iter().map(|i| {
            let mut m = Message::new(self.header.clone(), vec![i], vec![]);
            m.header.qd_count = 1;
            m.header.an_count = 0;
            m
        }).collect()
    }
    pub fn join(v: Vec<Self>) -> Self {
        MessageBuilder::new()
            .set_header(v[0].header.clone())
            .add_answers(v.iter().flat_map(|m| m.answers.clone()))
            .add_questions(v.iter().flat_map(|m| m.questions.clone()))
            .finish()

    }
    pub fn id(&self) -> u16 {
        self.header.id
    }

    pub fn opcode(&self) -> u8{
        self.header.opcode
    }

    pub fn rd(&self) -> bool {
        self.header.rd
    }

}


#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Answers(pub Vec<Answer>);

impl FromIterator<Answer> for Answers {
    fn from_iter<T: IntoIterator<Item=Answer>>(iter: T) -> Self {
        Self(iter.into_iter().collect::<Vec<_>>())
    }
}

impl Deref for Answers {
    type Target = Vec<Answer>;

    fn deref(&self) -> &Self::Target {
        & self.0
    }
}

impl DerefMut for Answers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl IntoIterator for Answers {
    type Item = Answer;
    type IntoIter = std::vec::IntoIter<Answer>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl Answers {

    pub fn from_questions<'a,Q: IntoIterator<Item = &'a Question>>(questions: Q) -> Self {
        questions.into_iter().map(|q| Answer::from_domain_name(&q.domain())).collect()
    }
    pub fn serialize(self) -> BytesMut {
        self.0.into_iter().flat_map(|a| a.serialize()).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Questions(Vec<Question>);

impl Deref for Questions {
    type Target = Vec<Question>;

    fn deref(&self) -> &Self::Target {
        & self.0
    }
}

impl DerefMut for Questions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl <'a >IntoIterator for &'a Questions {
    type Item = &'a Question;
    type IntoIter = std::slice::Iter<'a, Question>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for Questions {
    type Item = Question;
    type IntoIter = std::vec::IntoIter<Question>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Questions {
    pub fn serialize(self) -> BytesMut {
        self.0.into_iter().flat_map(|q| q.serialize()).collect()
    }
}

pub struct MessageBuilder {
    message: Message
}
#[allow(unused)]
impl MessageBuilder {
    pub fn new() -> Self {
        Self {message: Default::default()}
    }
    pub fn set_id(mut self, id:u16) -> Self {
        self.message.header.set_id(id);
        self
    }
    pub fn set_opcode(mut self, opcode: u8) -> Self {
        self.message.header.set_opcode(opcode);
        self
    }

    pub fn set_rd(mut self, rd: bool) -> Self {
        self.message.header.set_rd(rd);
        self
    }

    pub fn add_answer(mut self, answer: Answer) -> Self {
        self.message.answers.push(answer);
        self.message.header.increment_an_count();
        self
    }

    pub fn add_question(mut self, question: Question) -> Self {
        self.message.questions.push(question);
        self.message.header.increment_qd_count();
        self
    }

    pub fn set_header(mut self, header: Header) -> Self {
        self.message.header = header;
        self
    }

    pub fn add_questions<I: IntoIterator<Item = Question>>(mut self, iter: I) -> Self {
        for q in iter {
            self.message.questions.push(q);
            self.message.header.increment_qd_count();
        }
        self
    }

    pub fn add_answers<I: IntoIterator<Item = Answer>>(mut self, iter: I) -> Self {
        for a in iter {
            self.message.answers.push(a);
            self.message.header.increment_an_count();
        }
        self
    }
    pub fn finish(self) -> Message {
        self.message
    }
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
#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct Labels(Vec<Label>);
impl Deref for Labels {
    type Target = Vec<Label>;

    fn deref(&self) -> &Self::Target {
        & self.0
    }
}

impl Labels {
    pub fn parse(buf: &[u8], mut start: usize) -> (Labels, usize) {
        // println!("default start is {start}");
        let mut v : Labels = Default::default();
        loop {
            // println!("start is {start}");
            // println!("byte is {:<02x}", buf[start]);
            match buf[start] {
                zero if zero == b'\0' => {
                    if let Some(byte) = buf.get(start + 1) {
                      if *byte  == b'\0' {
                          start += 1;
                      }
                    };
                    break
                }
                x if x & 0b1100_0000 == 0b1100_0000 => {
                    let ptr = u16::from_be_bytes([
                        buf[start],
                        buf[start+1],
                    ]);
                    let ptr = ptr ^ 0b11_00_0000_0000_0000u16;
                    // println!("{ptr}");
                    v.extend_from_slice(Self::parse(buf, ptr as usize).0.iter().as_slice());
                    // println!("{:?}", v);
                    start += 2;
                },
                len => {
                    let end = start + len as usize + 1;
                    v.push(Label::from(&buf[start+1..end]));
                    start += len as usize + 1;
                }

            }
        }
        (v , start)

    }
}

impl DerefMut for Labels {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl Label {
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.val[0] as usize
    }



    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.val[1..]).to_string()
    }
}

impl Labels {

    pub fn to_string(&self) -> String {
        self.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(".")
    }
    pub fn from_bytes(seq: &[u8]) -> Self {
        let mut iter = seq.into_iter().copied();
        let mut vec = vec![];
        while let Some(len) = iter.next() {
            if len == b'0' {
                break
            }
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
        val.extend_from_slice(&value[..len as usize]);
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



#[cfg(test)]
mod tests{
    use bytes::{BufMut, BytesMut};
    use crate::answer::Answer;
    use crate::header::Header;
    use crate::message;
    use crate::message::{Label, Labels, Message, MessageBuilder};
    use crate::question::Question;

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

    #[test]
    fn test_builder() {
        let parsed = Header {
            id: 27901,
            qr: false,
            opcode: 0,
            aa: false,
            tc: false,
            rd: true,
            ra: false,
            reserved: 0,
            r_code: 0,
            qd_count: 1,
            an_count: 0,
            ns_count: 0,
            ar_count: 0,
        };
        let (id, opcode, rd) = parsed.get_id_opcode_rd();
        let message = MessageBuilder::new()
            .set_id(id)
            .set_opcode(opcode)
            .set_rd(rd)
            .finish();

        println!("{:#?}", message)
    }

    #[test]
    fn ser_de() {
        let message = message::MessageBuilder::new()
            .set_id(1488)
            .set_opcode(228)
            .set_rd(true)
            .finish();
        let ser = message.serialize();
        let de = Message::deserialize(&ser);
        println!("{:#?}", de);
    }

    #[test]
    fn label_to_string() {
        let label = Labels::from_domain("youtube.com");
        let v : String = label.to_string();
        assert_eq!(v.as_str(), "youtube.com")
    }

    #[test]
    fn test_multiple_ans() {
        let message = MessageBuilder::new()
            .set_id(1488)
            .add_answer(Answer::from_domain_name("hello.world"))
            .add_answer(Answer::from_domain_name("i.like.big.cocks"))
            .add_answer(Answer::from_domain_name("suck.some.dick"))
            .add_question(Question::from_domain_name("i.am.gay"))
            .finish();
        let cloned = message.clone();
        let from_iter = MessageBuilder::new()
            .set_id(228)
            .add_answers(cloned.answers)
            .finish();
        assert_eq!(from_iter.header.an(), 3);
        assert_eq!(message.answers, from_iter.answers)
    }

    #[test]
    fn testing_stuff() {
        let l = Labels::from_domain("youtube.com");
        let mut l = l.into_bytes_mut();
        l.put_u8(0);
        println!("{l:?}")
    }

    #[test]
    fn i_guess_it_broken() {
        let val = b"\x03abc\x11longassdomainname\x03com\0\x03def\xC0\x04\x05hello\0";
        let (l, end) = Labels::parse(val, 0);
        let _bytes = BytesMut::from(&val[end+1..]);
        let (a, b) = Labels::parse(val, end + 1);
        println!("{l:?}, rest is {:?}", a)
    }

    #[test]
    fn de(){
        let val = b"\xbf9\x01\0\0\x02\0\0\0\0\0\0\x03abc\x11longassdomainname\x03com\0\0\x01\0\x01\x03def\xc0\x10\0\x01\0\x01";
        let bytes = BytesMut::from(&val[49..]);
        println!("{:?}", bytes);
        let de = Message::deserialize(val);
        println!("{de:?}");
    }
}

