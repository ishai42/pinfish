//! This modules defines the constants and structures for encoding and
//! decoding NFS MOUNT protocol.
use crate::{nfs3::NfsFh3, xdr};
use pinfish_macros::{PackTo, UnpackFrom};

pub const PROGRAM: u32 = 100005;
pub const VERSION: u32 = 3;

pub const MOUNTPROC3_NULL: u32 = 0;
pub const MOUNTPROC3_MNT: u32 = 1;
pub const MOUNTPROC3_DUMP: u32 = 2;
pub const MOUNTPROC3_UMNT: u32 = 3;
pub const MOUNTPROC3_UMNTALL: u32 = 4;
pub const MOUNTPROC3_EXPORT: u32 = 5;

#[derive(PackTo, UnpackFrom, Debug)]
pub struct MountRes3Ok {
    pub handle: NfsFh3,
    pub auth_flavors: Vec<u32>,
}

pub type MountResult = Result<MountRes3Ok, u32>;
