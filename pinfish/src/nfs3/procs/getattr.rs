use crate::{
    nfs3::{FileAttributes, NfsFh3},
    result::Result,
    xdr,
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct GetAttr3Args {
    pub object: NfsFh3,
}

#[derive(PackTo, Debug, UnpackFrom)]
pub struct GetAttr3ResOk {
    pub attributes: FileAttributes,
}

pub type GetAttrResult = Result<GetAttr3ResOk>;
