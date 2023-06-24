use super::{StateId4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;


#[derive(PackTo, UnpackFrom, Debug)]
pub struct DelegReturn4Args {
    pub state_id: StateId4,
}


