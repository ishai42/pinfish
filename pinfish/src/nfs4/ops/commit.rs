use super::{Verifier4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Commit4Args {
    pub offset: u64,
    pub count: u32,
}

#[derive(PackTo, UnpackFrom, Debug, Clone)]
pub struct Commit4ResOk {
    pub verifier: Verifier4,
}
