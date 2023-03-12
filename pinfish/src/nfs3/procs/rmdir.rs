use crate::{
    nfs3::{DirOpArgs3, WccData},
    xdr,
};
use pinfish_macros::{PackTo, UnpackFrom};


#[derive(PackTo, Debug)]
pub struct Rmdir3Args {
    pub object: DirOpArgs3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Rmdir3ResOk {
    pub wcc_data: WccData,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Rmdir3ResFail {
    pub dir_wcc: WccData,
}

pub type RmdirResult = Result<Rmdir3ResOk, (u32, Rmdir3ResFail)>;
