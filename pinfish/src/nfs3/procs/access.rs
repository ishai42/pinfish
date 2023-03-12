use crate::{
    nfs3::{NfsFh3, PostOpAttributes},
    xdr::{self},
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct Access3Args {
    pub object: NfsFh3,
    pub access: u32,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Access3ResOk {
    pub obj_attributes: PostOpAttributes,
    pub access: u32,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Access3ResFail {
    pub dir_attributes: PostOpAttributes,
}

pub type AccessResult = Result<Access3ResOk, (u32, Access3ResFail)>;
