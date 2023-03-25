use super::{NfsFh4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;

#[derive(UnpackFrom, PackTo, Debug)]
pub struct GetFh4ResOk {
    pub object: NfsFh4,
}
