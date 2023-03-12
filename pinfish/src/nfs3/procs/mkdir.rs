use crate::{
    nfs3::{DirOpArgs3, PostOpAttributes, PostOpFh3, SetAttributes, WccData},
    xdr,
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct Mkdir3Args {
    pub mkdir_where: DirOpArgs3,
    pub attributes: SetAttributes,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Mkdir3ResOk {
    pub obj: PostOpFh3,
    pub attributes: PostOpAttributes,
    pub wcc_data: WccData,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Mkdir3ResFail {
    pub dir_wcc: WccData,
}

pub type MkdirResult = Result<Mkdir3ResOk, (u32, Mkdir3ResFail)>;
