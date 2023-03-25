use super::{NfsFh4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;

/// PUTFH
#[derive(PackTo, UnpackFrom, Debug)]
pub struct PutFh4Args {
    pub object: NfsFh4,
}
