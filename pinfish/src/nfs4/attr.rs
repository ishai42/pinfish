use crate::{
    result::Result,
    xdr::{self, PackTo, Packer, UnpackFrom, Unpacker},
};
use bytes::{Bytes, BytesMut};
use pinfish_macros::{PackTo, UnpackFrom};

pub const SUPPORTED_ATTRS: u32 = 0;
pub const TYPE: u32 = 1;
pub const FH_EXPIRE_TYPE: u32 = 2;
pub const CHANGE: u32 = 3;
pub const SIZE: u32 = 4;
// TODO 5-32
pub const MODE: u32 = 33;
// TODO 34
pub const OWNER: u32 = 36;
pub const OWNER_GROUP: u32 = 37;

/// A bitmap that serializes at NFS4 bitmap4 type
#[derive(PackTo, UnpackFrom, Debug)]
pub struct Bitmap4 {
    array: Vec<u32>,
}

impl Bitmap4 {
    /// Returns an empty bitmap
    fn new() -> Self {
        Bitmap4 { array: Vec::new() }
    }

    /// Checks if the `n`th bit is set
    fn is_set(&self, n: u32) -> bool {
        let word = (n / 32) as usize;
        let bit = n % 32;
        if word >= self.array.len() {
            false
        } else {
            (self.array[word] & (1 << bit)) != 0
        }
    }

    /// Sets the `n`th bit
    fn set(&mut self, n: u32) {
        let n = n as usize;
        let word = n / 32;
        let bit = n % 32;
        if word >= self.array.len() {
            self.array.resize(word + 1, 0);
        }

        self.array[word] |= 1 << bit;
    }

    /// Clears the `n`th bit
    fn clear(&mut self, n: u32) {
        let n = n as usize;
        let word = n / 32;
        let bit = n % 32;
        if word >= self.array.len() {
            return;
        }

        self.array[word] &= !(1 << bit);
    }
}

/// File types (RFC 7531)
#[derive(PackTo, Debug, UnpackFrom)]
pub enum NfsType4 {
    Reg = 1,
    Dir = 2,
    Blk = 3,
    Chr = 4,
    Lnk = 5,
    Sock = 6,
    Fifo = 7,
    AttrDir = 8,
    NamedAttr = 9,
}

#[derive(Debug)]
pub struct FileAttributes {
    pub supported_attrs: Option<Bitmap4>,
    pub obj_type: Option<NfsType4>,
    pub fh_expire_type: Option<u32>,
    pub change: Option<u64>,
    pub size: Option<u64>,
    pub mode: Option<u32>,
    pub owner: Option<u32>,
    pub owner_group: Option<u32>,
}

// applies macro to all fields in order
macro_rules! all_fields {
    ($macro:ident) => {
        $macro!(supported_attrs, SUPPORTED_ATTRS); // 0
        $macro!(obj_type, TYPE); // 1
        $macro!(fh_expire_type, FH_EXPIRE_TYPE); // 2
        $macro!(change, CHANGE); // 3
        $macro!(size, SIZE); // 4
        $macro!(mode, MODE); // 33
        $macro!(owner, OWNER); // 36
        $macro!(owner_group, OWNER_GROUP); // 37
    };
}

impl FileAttributes {
    /// returns a new, empty FileAttributes
    pub fn new() -> Self {
        FileAttributes {
            supported_attrs: None,
            obj_type: None,
            fh_expire_type: None,
            change: None,
            size: None,
            mode: None,
            owner: None,
            owner_group: None,
        }
    }

    /// builds a Bitmap4 corresponding to the attributes with values
    pub fn calculate_bitmap(&self) -> Bitmap4 {
        let mut bm = Bitmap4::new();

        macro_rules! set_bit {
            ($member:ident, $bit:expr) => {
                if let Some(_) = self.$member {
                    bm.set($bit);
                };
            };
        }

        all_fields!(set_bit);

        bm
    }
}

impl<B: Packer> PackTo<B> for FileAttributes {
    fn pack_to(&self, buf: &mut B) {
        let bm = self.calculate_bitmap();
        bm.pack_to(buf);

        let mut opaque = BytesMut::new();

        macro_rules! pack {
            ($member:ident, $bit:expr) => {
                if let Some(packme) = &self.$member {
                    packme.pack_to(&mut opaque)
                };
            };
        }

        all_fields!(pack);

        opaque.freeze().pack_to(buf);
    }
}

impl<B: Unpacker> UnpackFrom<B> for FileAttributes {
    fn unpack_from(buf: &mut B) -> Result<Self> {
        let bm = Bitmap4::unpack_from(buf)?;
        let mut result = FileAttributes::new();

        fn unpack_from<T: UnpackFrom<B>, B: Unpacker>(buf: &mut B) -> Result<T> {
            T::unpack_from(buf)
        }

        let mut opaque = Bytes::unpack_from(buf)?;

        macro_rules! unpack {
            ($member:ident, $bit:expr) => {
                if bm.is_set($bit) {
                    result.$member = Some(unpack_from(&mut opaque)?)
                }
            };
        }

        all_fields!(unpack);

        Ok(result)
    }
}
