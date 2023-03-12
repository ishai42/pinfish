use crate::{
    nfs3::{Count3, NfsFh3, Offset3, PostOpAttributes},
    xdr::{self},
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct Read3Args {
    pub file: NfsFh3,
    pub offset: Offset3,
    pub count: Count3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Read3ResOk {
    pub file_attributes: PostOpAttributes,
    pub count: Count3,
    pub eof: bool,
    pub data: bytes::Bytes,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Read3ResFail {
    pub file_attributes: PostOpAttributes,
}

pub type ReadResult = Result<Read3ResOk, (u32, Read3ResFail)>;
