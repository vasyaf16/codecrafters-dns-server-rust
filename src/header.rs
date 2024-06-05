#![allow(dead_code, unused)]

use std::fmt::Formatter;
use anyhow::bail;
use bytes::{BufMut, Bytes, BytesMut};


#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct Header {
    id: u16, // 16 bits
    qr: bool, // 1 bit
    opcode: u8, // 4 bits
    aa: bool, // 1 bit
    tc: bool, // 1 bit
    rd: bool, // 1 bit
    ra: bool, // 1 bit
    reserved: u8, // 3 bits
    r_code: u8, // 4 bits
    qd_count: u16, // 16 bits
    an_count: u16, // 16 bits
    ns_count: u16, // 16 bits
    ar_count: u16, // 16 bits
}
impl Default for Header {
    fn default() -> Self {
        Self::new(1234,true, 0,false,false,false,false,0,0,0,0,0,0)
    }
}

impl Header {
    pub fn new(id: u16, qr: bool, opcode: u8, aa: bool, tc: bool, rd: bool, ra: bool, reserved: u8, r_code: u8, qd_count: u16, an_count: u16, ns_count: u16, ar_count: u16) -> Self {
        Self {
            id,
            qr,
            opcode,
            aa,
            tc,
            rd,
            ra,
            reserved,
            r_code,
            qd_count,
            an_count,
            ns_count,
            ar_count,
        }
    }

    pub fn increment_qd_count(&mut self) {
        self.qd_count += 1;
    }
    pub fn deserialize(v: &[u8]) -> anyhow::Result<Self> {
        if v.len() != 12 {
            bail!("headers len should be exactly 12 bytes long")
        };
        let id = u16::from_be_bytes([v[0], v[1]]);
        let qr = v[2] >> 7 == 1;
        let opcode = v[2] >> 3 & 0b00001111;
        let aa = v[2] >> 2 & 1 == 1;
        let tc = v[2] >> 1 & 1 == 1;
        let rd = v[2] & 1 == 1;
        let ra = v[3] >> 7 == 1;
        let reserved = v[3] >> 4 & 0b00000_111;
        let r_code = v[3] & 0b0000_1111;
        let qd_count = u16::from_be_bytes([v[4], v[5]]);
        let an_count = u16::from_be_bytes([v[6], v[7]]);
        let ns_count = u16::from_be_bytes([v[8], v[9]]);
        let ar_count = u16::from_be_bytes([v[10], v[11]]);
        let header = Header {
            id,
            qr,
            opcode,
            aa,
            tc,
            rd,
            ra,
            reserved,
            r_code,
            qd_count,
            an_count,
            ns_count,
            ar_count,
        };
        Ok(header)
    }
    pub fn serialize(self) -> anyhow::Result<BytesMut> {
        let mut buffer = BytesMut::with_capacity(12);
        buffer.put_u16(self.id);
        let third_bite = 0u8
            | ((self.qr as u8) << 7)
            | (self.opcode << 3)
            | ((self.aa as u8) << 2)
            | ((self.tc as u8) << 1)
            | self.rd as u8;
        buffer.put_u8(third_bite);
        let fourth_bite = 0u8 | ((self.ra as u8) << 7) | self.reserved << 4 | self.r_code;
        buffer.put_u8(fourth_bite);
        buffer.put_u16(self.qd_count);
        buffer.put_u16(self.an_count);
        buffer.put_u16(self.ns_count);
        buffer.put_u16(self.ar_count);
        Ok(buffer)
    }
}


#[cfg(test)]
mod tests {
    use crate::header::Header;

    #[test]
    fn byte_shift() {
        let val = 128u8;
        let val2 = 0u8;
        assert_eq!(val >> 7, 1);
        assert_eq!(val2 >> 7, 0)
    }

    #[test]
    fn op_code_shift() {
        let val = 0b1101_1000u8;
        let e = val >> 3 & 0xF;
        assert_eq!(e, 0b00001011);
    }

    #[test]
    fn aa_shift() {
        let x = 0b11111_1_00u8;
        let e = x >> 2 & 1;
        let x2 = 0b1010_0_11u8;
        let e2 = x2 >> 2 & 1;
        assert_eq!(e, 1);
        assert_eq!(e2, 0);
    }

    #[test]
    fn tc_shift() {
        let x = 0b111111_1_0u8;
        let e = (x >> 1 & 1) == 1;
        let x2 = 0b101011_0_1u8;
        let e2 = (x2 >> 1 & 1) == 1;
        assert!(e);
        assert!(!e2)
    }

    #[test]
    fn rd_test() {
        let x = 0b1111_1111u8;
        let e = x & 1;
        let x2 = 0b1010_0100u8;
        assert_eq!(e, 1);
        assert_eq!(x2 & 1, 0);
    }

    #[test]
    fn reserved_test() {
        let x = 0b0_101_0011u8;
        let e = x >> 4 & 0b00000_111;
        assert_eq!(e, 0b00000_101);
    }

    #[test]
    fn r_code_test() {
        let x = 0b1010__1011u8;
        let e = x & 0b0000_1111;
        assert_eq!(e, 0b0000_1011)
    }

    #[test]
    fn deserializing_bytes() {
        let [first, second ] = 1488u16.to_be_bytes();
        let third = 0b1_1010_000u8;
        let fourth = 0b0_000_0000u8;
        let [fifth, sixth] = 3228u16.to_be_bytes();
        let [seventh, eight] = 2280u16.to_be_bytes();
        let [ninth, ten] = 1500u16.to_be_bytes();
        let [eleventh, twelve] = 5000u16.to_be_bytes();
        let bytes = [first, second, third, fourth, fifth, sixth, seventh, eight, ninth, ten, eleventh, twelve];
        let h = Header::deserialize(&bytes);
        println!("{:?}", h);
    }

    #[test]
    fn serialize() {
        let x = 0u8;
        let s = 0b0000_1010u8;
        let y = x | ((true as u8) << 7) | (s << 3) | ((false as u8) << 2) | ((true as u8) << 1) | 0;
        assert_eq!(y, 0b11010010)
    }

    #[test]
    fn serialize_and_deserialize() {
        let [first, second ] = 1488u16.to_be_bytes();
        let third = 0b1_1010_000u8;
        let fourth = 0b0_000_0000u8;
        let [fifth, sixth] = 3228u16.to_be_bytes();
        let [seventh, eight] = 2280u16.to_be_bytes();
        let [ninth, ten] = 1500u16.to_be_bytes();
        let [eleventh, twelve] = 5000u16.to_be_bytes();
        let bytes = [first, second, third, fourth, fifth, sixth, seventh, eight, ninth, ten, eleventh, twelve];
        let h = Header::deserialize(&bytes).unwrap();
        let result = h.serialize().unwrap();
        let bytes = bytes.to_vec();
        let result = result.to_vec();
        assert_eq!(bytes, result);

    }
}