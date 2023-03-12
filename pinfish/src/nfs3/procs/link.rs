use crate::{
    nfs3::{DirOpArgs3, NfsFh3, PostOpAttributes, WccData},
    xdr,
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct Link3Args {
    pub file: NfsFh3,
    pub link: DirOpArgs3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Link3ResOk {
    pub attributes: PostOpAttributes,
    pub linkdir_wcc: WccData,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Link3ResFail {
    pub attributes: PostOpAttributes,
    pub linkdir_wcc: WccData,
}

pub type LinkResult = Result<Link3ResOk, (u32, Link3ResFail)>;
