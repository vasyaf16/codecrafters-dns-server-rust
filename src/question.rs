
use bytes::{BufMut, BytesMut};
use nom::AsBytes;
use crate::message::{Class,Ty, Labels};
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Question {
    name: Labels,
    ty: Ty,
    class: Class,
}

impl Default for Question {
    fn default() -> Self {
        Self {
            name: Labels::from_bytes(b"\x0ccodecrafters\x02io"),
            ty: Ty::A,
            class: Class::IN,
        }
    }
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


}

#[cfg(test)]
mod test {
    use bytes::BytesMut;
    use crate::question::Question;

    #[test]
    fn test_serialize_question() {
        let question: Question = Default::default();
        let expected = b"\x0ccodecrafters\x02io\011";
        let _expected = BytesMut::from(&expected[..]);
        let got = question.serialize();
        println!("{:?}", got)
    }
}