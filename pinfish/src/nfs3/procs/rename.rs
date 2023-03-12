use crate::{
    nfs3::{DirOpArgs3, WccData},
    xdr,
};
use pinfish_macros::{PackTo, UnpackFrom};


#[derive(PackTo, Debug)]
pub struct Rename3Args {
    pub from: DirOpArgs3,
    pub to: DirOpArgs3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Rename3ResOk {
    pub fromdir_wcc: WccData,
    pub todir_wcc: WccData,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Rename3ResFail {
    pub fromdir_wcc: WccData,
    pub todir_wcc: WccData,
}

pub type RenameResult = Result<Rename3ResOk, (u32, Rename3ResFail)>;
