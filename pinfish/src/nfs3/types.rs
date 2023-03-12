use crate::xdr;
use pinfish_macros::{PackTo, UnpackFrom};

pub type Filename3 = String;
pub type NfsPath3 = String;
pub type FileId3 = u64;
pub type Cookie3 = u64;
// Used here for cookieverf3, createverf3, and writeverf3, all technically defiend
// as opaque[8]
pub type Verifier3 = u64;
pub type Uid3 = u32;
pub type Gid3 = u32;
pub type Size3 = u64;
pub type Count3 = u32;
pub type Mode3 = u32;
pub type Offset3 = u64;

#[derive(PackTo, Debug, UnpackFrom, Copy, Clone)]
pub enum FileType3 {
    Reg = 1,
    Dir = 2,
    Blk = 3,
    Chr = 4,
    Lnk = 5,
    Sock = 6,
    Fifo = 7,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct SpecData3 {
    pub data1: u32,
    pub data2: u32,
}

#[derive(PackTo, UnpackFrom, Debug, Clone, Default)]
pub struct NfsFh3 {
    pub data: Vec<u8>, // should be opaque<NFS3_FHSIZE>
}

/// The NfsTime3 gives the number of seconds and nano seconds since
/// midnight or zero hour January 1, 1970 Coordinated Universal Time
/// (UTC).
#[derive(PackTo, UnpackFrom, Debug)]
pub struct NfsTime3 {
    pub seconds: u32,
    pub nano_seconds: u32,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct FileAttributes {
    pub file_type: FileType3,
    pub mode: Mode3,
    pub num_links: u32,
    pub uid: Uid3,
    pub gid: Gid3,
    pub size: Size3,
    pub used: Size3,
    pub rdev: SpecData3,
    pub fsid: u64,
    pub file_id: FileId3,
    pub atime: NfsTime3,
    pub mtime: NfsTime3,
    pub ctime: NfsTime3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub enum TimeHow {
    DontChange,
    SetToServerTime,
    SetToClientTime(NfsTime3),
}

impl Default for TimeHow {
    fn default() -> Self {
        TimeHow::SetToServerTime
    }
}

#[derive(PackTo, UnpackFrom, Debug, Default)]
pub struct SetAttributes {
    pub mode: Option<Mode3>,
    pub uid: Option<Uid3>,
    pub gid: Option<Gid3>,
    pub size: Option<Size3>,
    pub atime: TimeHow,
    pub mtime: TimeHow,
}

/// Subset of pre-operation attributes used for weak cache consistency
#[derive(PackTo, UnpackFrom, Debug)]
pub struct WccAttributes {
    pub size: Size3,
    pub mtime: NfsTime3,
    pub ctime: NfsTime3,
}

pub type PostOpAttributes = Option<FileAttributes>;
pub type PreOpAttributes = Option<WccAttributes>;
pub type PostOpFh3 = Option<NfsFh3>;

#[derive(PackTo, UnpackFrom, Debug)]
pub struct DirOpArgs3 {
    pub dir: NfsFh3,
    pub name: Filename3,
}

/// Weak Cache Consistency data
#[derive(PackTo, UnpackFrom, Debug)]
pub struct WccData {
    pub before: PreOpAttributes,
    pub after: PostOpAttributes,
}
