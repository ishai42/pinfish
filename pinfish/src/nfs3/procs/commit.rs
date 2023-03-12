use crate::{
    nfs3::{Count3, NfsFh3, Offset3, Verifier3, WccData},
    xdr::{self},
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct Commit3Args {
    pub file: NfsFh3,
    pub offset: Offset3,
    pub count: Count3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Commit3ResOk {
    pub file_wcc: WccData,
    pub verifier: Verifier3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Commit3ResFail {
    file_wcc: WccData,
}

pub type CommitResult = Result<Commit3ResOk, (u32, Commit3ResFail)>;
