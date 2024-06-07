

use bytes::{BufMut, BytesMut};
use crate::message::{Class, Labels, Ty};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Answer {
    name: Labels,
    ty: Ty, // 16 bits
    class: Class, // 16 bits
    ttl: u32,
    rd_length: u16,
    r_data: Data, // 4 bits
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Data {
    A(u32),
}

impl Default for Answer {
    fn default() -> Self {
        let name = "codecrafters.io";
        let ty = 1;
        let class = 1;
        let ttl = 60;
        let data = 0x08080808u32;
        Self::new(name, ty, class, ttl, data)
    }
}

impl Answer {
    pub fn from_domain_name(name: &str) -> Self {
        let ty = 1;
        let class = 1;
        let ttl = 60;
        let data = 0x08080808u32;
        Self::new(name, ty, class, ttl, data)
    }


    pub fn new<A: AsRef<str>>(name: A, ty: u16, class: u16, ttl: u32, data: u32) -> Self {
        let name = Labels::from_domain(name.as_ref());
        let ty = Ty::try_from(ty).unwrap();
        let class = Class::try_from(class).unwrap();
        let r_data = match class {
            Class::IN => Data::A(data),
            _ => unimplemented!()
        };
        let rd_length = r_data.len();
        Self {
            name,
            ty,
            class,
            ttl,
            rd_length,
            r_data,
        }
    }
    pub fn serialize(self) -> BytesMut {
        let mut bytes = self.name.into_bytes_mut();
        bytes.put_u8(0);
        bytes.put_u16(self.ty as u16);
        bytes.put_u16(self.class as u16);
        bytes.put_u32(self.ttl);
        bytes.put_u16(self.rd_length);
        bytes.extend(self.r_data.as_bytes());
        bytes
    }

    pub fn deserialize(bytes: &[u8], start: usize) -> (Self, usize) {
        // let mut buf = bytes.into_iter().copied();
        // let labels = buf
        //     .by_ref()
        //     .take_while(|&c| {
        //         c != b'\0' })
        //     .collect::<BytesMut>();
        // let name = Labels::from_bytes(&labels);
        // let t = [buf.next().unwrap(), buf.next().unwrap()];
        // let ty = u16::from_be_bytes(t);
        // let ty = Ty::try_from(ty).unwrap();
        // let c = [buf.next().unwrap(), buf.next().unwrap()];
        // let class = u16::from_be_bytes(c);
        // let class = Class::try_from(class).unwrap();
        // let tl:[u8; 4] = buf.by_ref().take(4).collect::<Vec<_>>().try_into().unwrap();
        // let ttl = u32::from_be_bytes(tl);
        // let len = [buf.next().unwrap(), buf.next().unwrap()];
        // let rd_length = u16::from_be_bytes(len);
        // let data : [u8; 4] = buf.by_ref().take(rd_length as usize).collect::<Vec<_>>().try_into().unwrap();
        // let r_data = Data::A(u32::from_be_bytes(data));
        // let l = labels.len() + 1 + 2 + 2 + 4 + 2 + rd_length as usize; // 15
        let is_compressed = (bytes[start] & 0b1100_0000) == 0b1100_0000;
        let (labels, end) = if is_compressed {
            let offset = u16::from_be_bytes([bytes[start], bytes[start + 1]]);
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
        let ty = u16::from_be_bytes([bytes[end + 1], bytes[end + 2]]);
        let ty = Ty::try_from(ty).unwrap();
        let class = u16::from_be_bytes([bytes[end + 3], bytes[end + 4]]);
        let class = Class::try_from(class).unwrap();
        let ttl = u32::from_be_bytes([
            bytes[end + 5],
            bytes[end + 6],
            bytes[end + 7],
            bytes[end + 8],
        ]);
        let rd_length = u16::from_be_bytes([bytes[end + 9], bytes[end + 10]]);
        let r_data = Data::A(u32::from_be_bytes([
            bytes[end + 11],
            bytes[end + 12],
            bytes[end + 13],
            bytes[end + 14],
        ]));
        let l = end + 15;

        (Self {
            name,
            ty,
            class,
            ttl,
            rd_length,
            r_data,
        }, l)
    }
    #[allow(dead_code)]
    pub fn domain(&self) -> String {
        self.name.to_string()
    }
}

impl Data {
    pub fn len(&self) -> u16 {
        match *self {
            Data::A(_) => 4
        }
    }
    pub fn as_bytes(&self) -> BytesMut {
        match self {
            Data::A(a) => { BytesMut::from(&a.to_be_bytes()[..]) }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::answer::Answer;

    #[test]
    pub fn serialize_de() {
        let ans = Answer::from_domain_name("hello.world.io");
        let ser = ans.clone().serialize();
        let (de, _) = Answer::deserialize(&ser, 0);
        assert_eq!(ans, de);
    }
}

