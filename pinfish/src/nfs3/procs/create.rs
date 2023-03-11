use crate::{
    nfs3::{DirOpArgs3, PostOpAttributes, PostOpFh3, SetAttributes, Verifier3, WccData},
    xdr,
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub enum CreateHow3 {
    /// Create the file without checking for existence of a duplicate
    /// file in the same directory
    Unchecked(SetAttributes),
    /// Check if the file exists, operation will fail with NFS3ERR_EXIST if
    /// the file exists
    Guarded(SetAttributes),
    /// Use exclusive creation semantics
    Exclusive(Verifier3),
}

#[derive(PackTo, Debug)]
pub struct Create3Args {
    pub create_where: DirOpArgs3,
    pub how: CreateHow3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Create3ResOk {
    pub obj: PostOpFh3,
    pub attributes: PostOpAttributes,
    pub wcc_data: WccData,
}
