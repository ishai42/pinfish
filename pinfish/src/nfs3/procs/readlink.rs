use crate::{
    nfs3::{NfsFh3, PostOpAttributes},
    xdr::{self},
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct ReadLink3Args {
    pub symlink: NfsFh3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct ReadLink3ResOk {
    pub symlink_attributes: PostOpAttributes,
    pub data: String,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct ReadLink3ResFail {
    pub symlink_attributes: PostOpAttributes,
}

pub type ReadLinkResult = Result<ReadLink3ResOk, (u32, ReadLink3ResFail)>;
