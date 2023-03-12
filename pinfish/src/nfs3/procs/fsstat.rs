use crate::{
    nfs3::{NfsFh3, PostOpAttributes, Size3},
    xdr::{self},
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct Fsstat3Args {
    pub root: NfsFh3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Fsstat3ResOk {
    pub obj_attributes: PostOpAttributes,
    /// Total size in bytes of the file system
    pub tbytes: Size3,
    /// Free space in bytes.
    pub fbytes: Size3,
    /// Free space, in bytes, available to the user
    pub abytes: Size3,
    /// Total number of file slots
    pub tfiles: Size3,
    /// Number of free file slots
    pub ffiles: Size3,
    /// Number of free file slots available to the user
    pub afiles: Size3,
    /// A measure of file system volatility in seconds
    pub invarsec: u32,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Fsstat3ResFail {
    pub dir_attributes: PostOpAttributes,
}

pub type FsstatResult = Result<Fsstat3ResOk, (u32, Fsstat3ResFail)>;
