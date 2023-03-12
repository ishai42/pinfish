use crate::{
    nfs3::{NfsFh3, PostOpAttributes, },
    xdr::{self},
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct Pathconf3Args {
    pub root: NfsFh3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Pathconf3ResOk {
    pub obj_attributes: PostOpAttributes,
    pub linkmax: u32,
    pub name_max: u32,
    pub no_trunc: bool,
    pub chown_restricted: bool,
    pub case_insensitive: bool,
    pub case_preserving: bool,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Pathconf3ResFail {
    pub dir_attributes: PostOpAttributes,
}

pub type PathconfResult = Result<Pathconf3ResOk, (u32, Pathconf3ResFail)>;
