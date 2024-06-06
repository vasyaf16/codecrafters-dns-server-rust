
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

    pub fn from_domain_name(name: &str) -> Self {
        Self {
            name: Labels::from_domain(name),
            ty: Ty::A,
            class : Class::IN
        }
    }
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
        buf.extend(v.iter().flat_map(|l| l.as_bytes()));
        buf.put_u8(0);
        buf.put_u16(self.ty as u16);
        buf.put_u16(self.class as u16);
        buf
    }

    pub fn deserialize_inner(bytes: &[u8]) -> (Self, usize) {
        let mut buf = bytes.into_iter().copied();
        let labels = buf
            .by_ref()
            .take_while(|&c| {
                c != b'\0' })
            .collect::<BytesMut>();
        let name = Labels::from_bytes(&labels);
        let t = [buf.next().unwrap(), buf.next().unwrap()];
        let ty = u16::from_be_bytes(t);
        let ty = Ty::try_from(ty).unwrap();
        let c = [buf.next().unwrap(), buf.next().unwrap()];
        let class = u16::from_be_bytes(c);
        let class = Class::try_from(class).unwrap();
        let len = labels.len() + 5;
        (Self {
            name,
            ty,
            class
        }, len )
    }

    pub fn deserialize(bytes : &[u8]) -> Self {
        Self::deserialize_inner(bytes).0
    }

    pub fn domain(&self) -> String {
        self.name.to_string()
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

    #[test]
    fn test_deserialize_question() {
        let val = b"\x0ccodecrafters\x02io\0\x00\x01\x00\x01somebullshit";
        let q = Question::deserialize_inner(val);
        println!("{:#?}", q)

    }
}