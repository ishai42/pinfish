use crate::{
    nfs3::{DirOpArgs3, NfsFh3, PostOpAttributes},
    xdr::{self},
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct Lookup3Args {
    pub what: DirOpArgs3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Lookup3ResOk {
    pub object: NfsFh3,
    pub obj_attributes: PostOpAttributes,
    pub dir_attributes: PostOpAttributes,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Lookup3ResFail {
    pub dir_attributes: PostOpAttributes,
}

pub type LookupResult = Result<Lookup3ResOk, (u32, Lookup3ResFail)>;
