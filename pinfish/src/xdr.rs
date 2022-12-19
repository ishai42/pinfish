/// This module implements helper traits for packing and unpacking
/// packets in XDR standard (RFC 4506)
use bytes::{Buf, BufMut, Bytes};

const PAD_ZERO: [u8; 4] = [0; 4];

/// A trait for packing data in XDR format into a buffer.
pub trait Packer {
    fn pack_uint(&mut self, value: u32);

    fn pack_int(&mut self, value: i32);

    fn pack_hyper(&mut self, value: i64);

    fn pack_uhyper(&mut self, value: u64);

    fn pack_bool(&mut self, value: bool);

    fn pack_float(&mut self, value: f32);

    fn pack_double(&mut self, value: f64);

    fn pack_opaque(&mut self, value: &[u8]);

    fn pack_opaque_fixed(&mut self, value: &[u8]);

    fn pack_string(&mut self, value: &str);

    fn pack_array<I, F>(&mut self, array: &[I], pack_fn: F)
    where
        F: Fn(&mut Self, &I) -> (),
    {
        self.pack_uint(array.len() as u32);
        for item in array {
            pack_fn(self, item);
        }
    }
}

impl<Buffer: BufMut> Packer for Buffer {
    #[inline]
    fn pack_uint(&mut self, value: u32) {
        self.put_u32(value)
    }

    #[inline]
    fn pack_int(&mut self, value: i32) {
        self.put_i32(value)
    }

    #[inline]
    fn pack_hyper(&mut self, value: i64) {
        self.put_i64(value)
    }

    #[inline]
    fn pack_uhyper(&mut self, value: u64) {
        self.put_u64(value)
    }

    #[inline]
    fn pack_bool(&mut self, value: bool) {
        self.put_u32(value as u32)
    }

    #[inline]
    fn pack_float(&mut self, value: f32) {
        self.put_f32(value)
    }

    #[inline]
    fn pack_double(&mut self, value: f64) {
        self.put_f64(value)
    }

    #[inline]
    fn pack_opaque(&mut self, value: &[u8]) {
        self.put_u32(value.len() as u32);
        self.pack_opaque_fixed(value);
    }

    #[inline]
    fn pack_opaque_fixed(&mut self, value: &[u8]) {
        let len = value.len();
        self.put_slice(value);
        self.put_slice(&PAD_ZERO[..(4 - len % 4) % 4])
    }

    #[inline]
    fn pack_string(&mut self, value: &str) {
        self.pack_opaque(value.as_bytes())
    }
}

/// A trait for unpacking XDR from a buffer
pub trait Unpacker {
    fn unpack_uint(&mut self) -> u32;

    fn unpack_int(&mut self) -> i32;

    fn unpack_hyper(&mut self) -> i64;

    fn unpack_uhyper(&mut self) -> u64;

    fn unpack_bool(&mut self) -> bool;

    fn unpack_float(&mut self) -> f32;

    fn unpack_double(&mut self) -> f64;

    fn unpack_opaque(&mut self) -> bytes::Bytes;

    fn unpack_opaque_fixed(&mut self, nbytes: usize) -> bytes::Bytes;

    fn unpack_vec<I, F>(&mut self, unpack_fn: F) -> Vec<I>
    where
        F: Fn(&mut Self) -> I,
    {
        let len = self.unpack_uint() as usize;
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(unpack_fn(self));
        }

        result
    }
}

impl<Buffer: Buf> Unpacker for Buffer {
    #[inline]
    fn unpack_uint(&mut self) -> u32 {
        self.get_u32()
    }

    #[inline]
    fn unpack_int(&mut self) -> i32 {
        self.get_i32()
    }

    #[inline]
    fn unpack_hyper(&mut self) -> i64 {
        self.get_i64()
    }

    #[inline]
    fn unpack_uhyper(&mut self) -> u64 {
        self.get_u64()
    }

    #[inline]
    fn unpack_bool(&mut self) -> bool {
        self.unpack_uint() != 0
    }

    #[inline]
    fn unpack_float(&mut self) -> f32 {
        self.get_f32()
    }

    #[inline]
    fn unpack_double(&mut self) -> f64 {
        self.get_f64()
    }

    #[inline]
    fn unpack_opaque(&mut self) -> bytes::Bytes {
        let len = self.unpack_uint() as usize;
        self.unpack_opaque_fixed(len)
    }

    #[inline]
    fn unpack_opaque_fixed(&mut self, nbytes: usize) -> bytes::Bytes {
        let ret = self.copy_to_bytes(nbytes);
        self.advance((4 - nbytes % 4) % 4);
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack() {
        let mut buf = bytes::BytesMut::new();

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

        let mut buf = buf.freeze();

        assert_eq!(buf.unpack_uint(), 0x01020304);
        assert_eq!(buf.unpack_uhyper(), 0x0506070809101112);
        assert_eq!(buf.unpack_int(), -1234567);
        assert_eq!(buf.unpack_hyper(), -1234567890111213);
        assert_eq!(buf.unpack_bool(), true);
        assert_eq!(buf.unpack_bool(), false);
        assert_eq!(buf.unpack_float(), 0.1234);
        assert_eq!(buf.unpack_double(), 0.5678);
        assert_eq!(
            buf.unpack_opaque_fixed(5).as_ref(),
            &[0x14, 0x15, 0x16, 0x17, 0x18]
        );
        assert_eq!(
            buf.unpack_opaque().as_ref(),
            &[0x19, 0x20, 0x21, 0x22, 0x23]
        );
        assert_eq!(
            buf.unpack_opaque().as_ref(),
            b"The quick brown fox jumps over the lazy dog"
        );
    }
}

/// Trait that allows packing objects into a buffer.
pub trait PackTo<B> {
    /// Pack `self` into `buf`
    fn pack_to(&self, buf: &mut B);
}

/// Trait that allows unpacking objects from a buffer
pub trait UnpackFrom<B> {
    fn unpack_from(buf: &mut B) -> Self;
}

/// Allow generic `Vec<T>` implementation of `PackTo` and `UnpackFrom` for the type
/// if the traits are implemented for `T`
pub(crate) trait VecPackUnpack {}


macro_rules! impl_pack_to (
    ($type:ty, $method:ident) => {
        impl VecPackUnpack for $type {
        }

        impl<B: Packer> PackTo<B> for $type {
            fn pack_to(&self, buf: &mut B) {
                buf.$method(*self)
            }
        }
    }
);

macro_rules! impl_unpack_from (
    ($type:ty, $method:ident) => {
        impl<B: Unpacker> UnpackFrom<B> for $type {
            fn unpack_from(buf: &mut B) -> Self {
                buf.$method()
            }
        }
    }
);

// Note: explicitly NOT implemented for u8 to allow trait implementation
// for Vec<u8> and a generic Vec<T>.  XDR does not define encoding for "byte"
// so it would have to be encoded as 4-byte unsigned int which is not what's
// expected for a byte vector.
impl_pack_to!(u32, pack_uint);
impl_pack_to!(i32, pack_int);
impl_pack_to!(i64, pack_hyper);
impl_pack_to!(u64, pack_uhyper);
impl_pack_to!(bool, pack_bool);
impl_pack_to!(f32, pack_float);
impl_pack_to!(f64, pack_double);
impl_pack_to!(&str, pack_string);

// Note: explicitly NOT implemented for u8 to allow trait implementation
// for Vec<u8> and a generic Vec<T>.  XDR does not define encoding for "byte"
// so it would have to be encoded as 4-byte unsigned int which is not what's
// expected for a byte vector.
impl_unpack_from!(u32, unpack_uint);
impl_unpack_from!(i32, unpack_int);
impl_unpack_from!(i64, unpack_hyper);
impl_unpack_from!(u64, unpack_uhyper);
impl_unpack_from!(bool, unpack_bool);
impl_unpack_from!(f32, unpack_float);
impl_unpack_from!(f64, unpack_double);
impl_unpack_from!(bytes::Bytes, unpack_opaque);

impl<B: Packer> PackTo<B> for String {
    fn pack_to(&self, buf: &mut B) {
        buf.pack_string(self);
    }
}

impl<T: PackTo<B>, B: Packer> PackTo<B> for Option<T> {
    fn pack_to(&self, buf: &mut B) {
        match self {
            Some(t) => {
                buf.pack_bool(true);
                t.pack_to(buf);
            }
            None => {
                buf.pack_bool(false);
            }
        }
    }
}

impl<B: Packer> PackTo<B> for Vec<u8> {
    fn pack_to(&self, buf: &mut B) {
        buf.pack_opaque(self);
    }
}

impl<B: Packer> PackTo<B> for Bytes {
    fn pack_to(&self, buf: &mut B) {
        buf.pack_opaque(self.as_ref());
    }
}


impl<B: Packer> PackTo<B> for [u8; 16] {
    fn pack_to(&self, buf: &mut B) {
        buf.pack_opaque_fixed(self);
    }
}

impl<T: std::fmt::Debug + VecPackUnpack + PackTo<B>, B: Packer> PackTo<B> for Vec<T> {
    fn pack_to(&self, buf: &mut B) {
        let len = self.len() as u32;
        buf.pack_uint(len);
        for item in self.iter() {
            item.pack_to(buf);
        }
    }
}

impl<T: std::fmt::Debug + VecPackUnpack + UnpackFrom<B>, B: Unpacker> UnpackFrom<B> for Vec<T> {
    fn unpack_from(buf: &mut B) -> Self {
        let len = buf.unpack_uint() as usize;
        let mut result = Vec::with_capacity(len.into());
        for _ in 0..len {
            result.push(T::unpack_from(buf))
        }

        result
    }
}

impl<T: UnpackFrom<B>, B: Unpacker> UnpackFrom<B> for Option<T> {
    fn unpack_from(buf: &mut B) -> Self {
        let n = u32::unpack_from(buf);
        match n {
            0 => None,
            1 => Some(T::unpack_from(buf)),
            _ => todo!("error handling"),
        }
    }
}

impl<B: Unpacker> UnpackFrom<B> for Vec<u8> {
    fn unpack_from(buf: &mut B) -> Self {
        let bytes = buf.unpack_opaque();
        bytes.into_iter().collect()
    }
}

impl<B: Unpacker> UnpackFrom<B> for String {
    fn unpack_from(buf: &mut B) -> Self {
        let v = Vec::<u8>::unpack_from(buf);
        String::from_utf8(v).unwrap()
    }
}
