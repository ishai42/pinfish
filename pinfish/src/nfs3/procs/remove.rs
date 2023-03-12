use crate::{
    nfs3::{DirOpArgs3, WccData},
    xdr,
};
use pinfish_macros::{PackTo, UnpackFrom};


#[derive(PackTo, Debug)]
pub struct Remove3Args {
    pub object: DirOpArgs3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Remove3ResOk {
    pub wcc_data: WccData,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Remove3ResFail {
    pub dir_wcc: WccData,
}

pub type RemoveResult = Result<Remove3ResOk, (u32, Remove3ResFail)>;
