use bytes::BytesMut;
use crate::message::{Class, Label, Ty};

pub struct Answer {
    name: Vec<Label>,
    ty: Ty,
    class: Class,
    ttl: u32,
    rd_length: u16,
    r_data: Data
}
#[non_exhaustive]
pub enum Data {
    A(u32)
}