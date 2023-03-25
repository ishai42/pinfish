use super::{Component4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;

/// LOOKUP operation arguments.  The directory is
/// passed as current FH
#[derive(PackTo, UnpackFrom, Debug)]
pub struct Lookup4Args {
    pub objname: Component4,
}
