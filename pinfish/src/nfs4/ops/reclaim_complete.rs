use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;

#[derive(UnpackFrom, PackTo, Debug)]
pub struct ReclaimComplete4Args {
    pub one_fs: bool,
}
