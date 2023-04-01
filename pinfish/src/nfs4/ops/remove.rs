use super::{Component4, ChangeInfo4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Remove4Args {
    pub target: Component4,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Remove4ResOk {
    pub change_info: ChangeInfo4,
}
