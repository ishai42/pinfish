use crate::{
    nfs3::{Count3, NfsFh3, Offset3, Verifier3, WccData},
    xdr::{self},
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug, UnpackFrom)]
pub enum StableHow {
    Unstable,
    DataSync,
    FileSync,
}

#[derive(PackTo, Debug)]
pub struct Write3Args {
    pub file: NfsFh3,
    pub offset: Offset3,
    pub count: Count3,
    pub stable: StableHow,
    pub data: bytes::Bytes,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Write3ResOk {
    pub file_wcc: WccData,
    pub count: Count3,
    pub committed: StableHow,
    pub verifier: Verifier3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Write3ResFail {
    file_wcc: WccData,
}

pub type WriteResult = Result<Write3ResOk, (u32, Write3ResFail)>;
