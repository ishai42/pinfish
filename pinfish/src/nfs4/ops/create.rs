use super::{SpecData4, Bitmap4, FileAttributes, ChangeInfo4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;

#[derive(PackTo, Debug)]
pub enum CreateType4 {
    /// Symbolic link
    #[xdr(5)]
    Link(String),
    /// Block device
    #[xdr(3)]
    Block(SpecData4),
    /// Char device
    #[xdr(4)]
    Char(SpecData4),
    /// Socket
    #[xdr(6)]
    Socket,
    /// FIFO
    #[xdr(7)]
    Fifo,
    /// Directory
    #[xdr(2)]
    Directory,
}

#[derive(PackTo, Debug)]
pub struct Create4Args {
    pub objtype: CreateType4,
    pub component: String,
    pub attributes: FileAttributes,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct Create4ResOk {
    change_info: ChangeInfo4,
    attr_set: Bitmap4,
}

