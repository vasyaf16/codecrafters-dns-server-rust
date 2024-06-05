use bytes::{BufMut, BytesMut};

pub struct Question {
    name: BytesMut,
    ty: u16,
    class: u16,
}
#[allow(dead_code)]
impl Question {
    pub fn new(buf: &[u8], ty: u16, class: u16) -> Self {
        Self {
            name: BytesMut::from(buf),
            ty,
            class,
        }
    }
    pub fn serialize(self) -> BytesMut {
        let mut buf = self.name;
        buf.put_u8(0);
        buf.put_u16(self.ty);
        buf.put_u16(self.class);
        buf
    }

    pub fn for_question_test() -> Self {
        Self {
            name: b"\x0ccodecrafters\x02io"[..].into(),
            ty: 1,
            class: 1,
        }
    }
}