use crate::{
    nfs3::{DirOpArgs3, PostOpAttributes, PostOpFh3, SetAttributes, WccData, NfsPath3},
    xdr,
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct SymLinkData3 {
    pub attributes: SetAttributes,
    pub data: NfsPath3,
}

#[derive(PackTo, Debug)]
pub struct SymLink3Args {
    pub symlink_where: DirOpArgs3,
    pub data: SymLinkData3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct SymLink3ResOk {
    pub obj: PostOpFh3,
    pub attributes: PostOpAttributes,
    pub wcc_data: WccData,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct SymLink3ResFail {
    pub dir_wcc: WccData,
}

pub type SymLinkResult = Result<SymLink3ResOk, (u32, SymLink3ResFail)>;
