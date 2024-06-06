
use bytes::{BufMut, BytesMut};
use nom::AsBytes;
use crate::message::{Class,Ty, Labels};

pub struct Question {
    name: Labels,
    ty: Ty,
    class: Class,
}
#[allow(dead_code)]
impl Question {
    pub fn new(buf: &[u8], ty: u16, class: u16) -> Self {
        Self {
            name: Labels::from_bytes(buf),
            ty: ty.try_into().unwrap(),
            class: class.try_into().unwrap(),
        }
    }
    pub fn serialize(self) -> BytesMut {
        let mut buf = BytesMut::new();
        let v = self.name;
        // let _val = v.into_iter().flat_map(|l| l.as_bytes()).collect::<Vec<_>>();
        buf.extend(v.iter().flat_map(|l| l.as_bytes()));
        buf.put_u8(0);
        buf.put_u16(self.ty as u16);
        buf.put_u16(self.class as u16);
        buf
    }

    pub fn for_question_test() -> Self {
        Self {
            name: Labels::from_bytes(b"\x0ccodecrafters\x02io"),
            ty: Ty::A,
            class: Class::IN,
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;
    use crate::question::Question;

    #[test]
    fn test_serialize_question() {
        let question = Question::for_question_test();
        let expected = b"\x0ccodecrafters\x02io\011";
        let _expected = BytesMut::from(&expected[..]);
        let got = question.serialize();
        println!("{:?}", got)
    }
}