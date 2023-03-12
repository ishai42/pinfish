use crate::{
    nfs3::{NfsFh3, NfsTime3, SetAttributes, WccData},
    xdr,
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct SetAttr3Args {
    pub object: NfsFh3,
    pub new_attributes: SetAttributes,
    /// Guard, if present is compared to object ctime
    pub guard: Option<NfsTime3>,
}

#[derive(PackTo, Debug, UnpackFrom)]
pub struct SetAttr3ResOk {
    pub obj_wcc: WccData,
}

#[derive(PackTo, Debug, UnpackFrom)]
pub struct SetAttr3ResFail {
    pub obj_wcc: WccData,
}

pub type SetAttrResult = Result<SetAttr3ResOk, (u32, SetAttr3ResFail)>;
