use super::{StateId4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;

#[derive(PackTo, Debug)]
pub struct Read4Args {
    pub state_id: StateId4,
    pub offset: u64,
    pub count: u32,
}

#[derive(UnpackFrom, PackTo, Debug, Clone)]
pub struct Read4ResOk {
    pub eof: bool,
    pub data: bytes::Bytes,
}
