use crate::{
    nfs3::{NfsFh3, PostOpAttributes, NfsTime3, Size3},
    xdr::{self},
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct Fsinfo3Args {
    pub root: NfsFh3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Fsinfo3ResOk {
    pub obj_attributes: PostOpAttributes,
    pub rtmax: u32,
    pub rtperf: u32,
    pub rtmult: u32,
    pub wtmax: u32,
    pub wtpref: u32,
    pub wtmult: u32,
    pub dtperf: u32,
    pub maxfilesize: Size3,
    pub time_delta: NfsTime3,
    pub properties: u32,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Fsinfo3ResFail {
    pub dir_attributes: PostOpAttributes,
}

pub type FsinfoResult = Result<Fsinfo3ResOk, (u32, Fsinfo3ResFail)>;
