#![allow(dead_code, unused)]

use bytes::{BufMut, BytesMut};
use nom::character::complete::u32;
use crate::message::{Class, Label, Labels, Ty};
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Answer {
    name: Labels,
    ty: Ty, // 16 bits
    class: Class, // 16 bits
    ttl: u32,
    rd_length: u16,
    r_data: Data // 4 bits
}
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Data {
    A(u32),
}

impl Answer {

    pub fn for_third_test() -> Self {
        let name = "codecrafters.io";
        let ty = 1;
        let class = 1;
        let ttl = 60;
        let data = 0x08080808u32;
        Self::new(name,ty,class,ttl,data)
    }

    pub fn new<A: AsRef<str>>(name: A, ty: u16, class: u16, ttl: u32, data: u32) -> Self {
        let name = Labels::from_domain(name.as_ref());
        let ty = Ty::try_from(ty).unwrap();
        let class = Class::try_from(class).unwrap();
        let r_data = match class {
            Class::IN => Data::A(data),
            _ => unimplemented!()
        };
        let rd_length = r_data.len() as u16;
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
        bytes.put_u16(self.ty as u16);
        bytes.put_u16(self.class as u16);
        bytes.put_u32(self.ttl);
        bytes.put_u16(self.rd_length);
        let rdata = self.r_data.as_bytes();
        println!("rdata len is {:?}", rdata.len());
        bytes.put_u32(u32::from_le_bytes([rdata[0],rdata[1], rdata[2], rdata[3]]));
        bytes
    }
}

impl Data {

    pub fn len(&self) -> usize {
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

