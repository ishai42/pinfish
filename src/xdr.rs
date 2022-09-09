use bytes::{Buf, BufMut};

const PAD_ZERO: [u8; 4] = [0; 4];

/// Packer object that knows how to pack the basic XDR types into
/// a buffer
struct Packer<Buffer: BufMut> {
    buf: Buffer,
}

impl<Buffer: BufMut> Packer<Buffer> {
    pub fn new(buf: Buffer) -> Self {
        Packer { buf }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut Buffer {
        &mut self.buf
    }

    #[inline]
    pub fn get_ref(&self) -> &Buffer {
        &self.buf
    }

    #[inline]
    pub fn into_inner(self) -> Buffer {
        self.buf
    }

    #[inline]
    pub fn pack_uint(&mut self, value: u32) {
        self.buf.put_u32(value)
    }

    #[inline]
    pub fn pack_int(&mut self, value: i32) {
        self.buf.put_i32(value)
    }

    #[inline]
    pub fn pack_hyper(&mut self, value: i64) {
        self.buf.put_i64(value)
    }

    #[inline]
    pub fn pack_uhyper(&mut self, value: u64) {
        self.buf.put_u64(value)
    }

    #[inline]
    pub fn pack_bool(&mut self, value: bool) {
        self.buf.put_u32(value as u32)
    }

    #[inline]
    pub fn pack_float(&mut self, value: f32) {
        self.buf.put_f32(value)
    }

    #[inline]
    pub fn pack_double(&mut self, value: f64) {
        self.buf.put_f64(value)
    }

    #[inline]
    pub fn pack_opaque(&mut self, value: &[u8]) {
        self.buf.put_u32(value.len() as u32);
        self.pack_opaque_fixed(value);
    }

    #[inline]
    pub fn pack_opaque_fixed(&mut self, value: &[u8]) {
        let len = value.len();
        self.buf.put_slice(value);
        self.buf.put_slice(&PAD_ZERO[..(4 - len % 4) % 4])
    }

    #[inline]
    pub fn pack_string(&mut self, value: &str) {
        self.pack_opaque(value.as_bytes())
    }
}

/// Packer object that knows how to pack the basic XDR types into
/// a buffer
struct Unpacker<Buffer: Buf> {
    buf: Buffer,
}

impl<Buffer: Buf> Unpacker<Buffer> {
    pub fn new(buf: Buffer) -> Self {
        Unpacker { buf }
    }

    #[inline]
    pub fn unpack_uint(&mut self) -> u32 {
        self.buf.get_u32()
    }

    #[inline]
    pub fn unpack_int(&mut self) -> i32 {
        self.buf.get_i32()
    }

    #[inline]
    pub fn unpack_hyper(&mut self) -> i64 {
        self.buf.get_i64()
    }

    #[inline]
    pub fn unpack_uhyper(&mut self) -> u64 {
        self.buf.get_u64()
    }

    #[inline]
    pub fn unpack_bool(&mut self) -> bool {
        self.unpack_uint() != 0
    }

    #[inline]
    pub fn unpack_float(&mut self) -> f32 {
        self.buf.get_f32()
    }

    #[inline]
    pub fn unpack_double(&mut self) -> f64 {
        self.buf.get_f64()
    }

    #[inline]
    pub fn unpack_opaque(&mut self) -> bytes::Bytes {
        let len = self.unpack_uint() as usize;
        self.unpack_opaque_fixed(len)
    }

    #[inline]
    pub fn unpack_opaque_fixed(&mut self, nbytes: usize) -> bytes::Bytes {
        let ret = self.buf.copy_to_bytes(nbytes);
        self.buf.advance((4 - nbytes % 4) % 4);
        ret
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack() {
        let buf = bytes::BytesMut::new();
        let mut buf = Packer::new(buf);

        buf.pack_uint(0x01020304);
        buf.pack_uhyper(0x0506070809101112);
        buf.pack_int(-1234567);
        buf.pack_hyper(-1234567890111213);
        buf.pack_bool(true);
        buf.pack_bool(false);
        buf.pack_float(0.1234);
        buf.pack_double(0.5678);
        buf.pack_opaque_fixed(&[0x14, 0x15, 0x16, 0x17, 0x18]);
        buf.pack_opaque(&[0x19, 0x20, 0x21, 0x22, 0x23]);
        buf.pack_string("The quick brown fox jumps over the lazy dog");

        let buf = buf.into_inner().freeze();
        let mut buf = Unpacker::new(buf);
        assert_eq!(buf.unpack_uint(), 0x01020304);
        assert_eq!(buf.unpack_uhyper(), 0x0506070809101112);
        assert_eq!(buf.unpack_int(), -1234567);
        assert_eq!(buf.unpack_hyper(), -1234567890111213);
        assert_eq!(buf.unpack_bool(), true);
        assert_eq!(buf.unpack_bool(), false);
        assert_eq!(buf.unpack_float(), 0.1234);
        assert_eq!(buf.unpack_double(), 0.5678);
        assert_eq!(buf.unpack_opaque_fixed(5).as_ref(), &[0x14, 0x15, 0x16, 0x17, 0x18]);
        assert_eq!(buf.unpack_opaque().as_ref(), &[0x19, 0x20, 0x21, 0x22, 0x23]);
        assert_eq!(buf.unpack_opaque().as_ref(), b"The quick brown fox jumps over the lazy dog");
    }

    #[test]
    fn test_get_xxx() {
        let buf = bytes::BytesMut::new();
        let mut buf = Packer::new(buf);

        buf.pack_uint(0x12345678);

        {
            let borrow_mut : &mut bytes::BytesMut = buf.get_mut();
            borrow_mut[0] = 0xab;
        }

        {
            let borrow : &bytes::BytesMut = buf.get_ref();
            assert_eq!(borrow[1], 0x34_u8);
            assert_eq!(borrow[0], 0xab_u8);
        }

    }

}
