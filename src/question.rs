use bytes::{BufMut, BytesMut};
use nom::AsBytes;
use crate::message::{Class, Ty, Labels};

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
            class: Class::IN,
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

    pub fn deserialize(bytes: &[u8], start: usize) -> (Self, usize) {


        let is_compressed = (bytes[start] & 0b1100_0000) == 0b1100_0000;
        let (labels, end) = if is_compressed {
                let offset = u16::from_be_bytes([bytes[start], bytes[start+1]]);
                let offset = offset ^ 0b1100_0000_0000_0000u16;
                let l = bytes[offset as usize..]
                    .iter()
                    .copied()
                    .take_while(|&c| {
                        c != b'\0'
                    })
                    .collect::<BytesMut>();
                let e = start + 1;
            (l, e)
        } else {
            let l =
                bytes[start..]
                    .iter()
                    .copied()
                    .take_while(|&c| {
                        c != b'\0'
                    })
                    .collect::<BytesMut>();
            let e = start + l.len();
            (l, e)
        };
        let name = Labels::from_bytes(&labels);
        let t = [bytes[end+1], bytes[end+2]];
        let ty = u16::from_be_bytes(t);
        let ty = Ty::try_from(ty).unwrap();
        let c = [bytes[end+3], bytes[end+4]];
        let class = u16::from_be_bytes(c);
        let class = Class::try_from(class).unwrap();
        let len = end + 5;
        (Self {
            name,
            ty,
            class,
        }, len)
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
        let val = b"\x0ccodecrafters\x02io\0\x00\x01\x00\x01\xC0\x00\x00\x01\x00\x01";
        let (_q, s) = Question::deserialize(val, 0);
        let (_q, e) = Question::deserialize(val, s);
        assert_eq!(e, val.len())
    }

    #[test]
    fn test_xor() {
        let one = 0b11_0000_00_0000_0001u16;
        let two = 0b11_0000_00_0000_0010u16;
        let three = 0b11_0000_00_0000_0011u16;
        let four = 0b11_0000_00_0000_0100u16;
        let ten = 0b11_0000_00_0000_1010u16;
        let xor1 = one ^ 0b1100_0000_0000_0000u16;
        let xor2 = two ^ 0b1100_0000_0000_0000u16;
        let xor3 = three ^ 0b1100_0000_0000_0000u16;
        let xor4 = four ^ 0b1100_0000_0000_0000u16;
        let xor5 = ten ^ 0b1100_0000_0000_0000u16;
        assert_eq!(xor1, 1);
        assert_eq!(xor2, 2);
        assert_eq!(xor3, 3);
        assert_eq!(xor4, 4);
        assert_eq!(xor5, 10)
    }
}