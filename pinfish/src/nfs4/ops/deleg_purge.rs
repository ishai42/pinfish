use super::{ClientId4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;


#[derive(PackTo, UnpackFrom, Debug)]
pub struct DelegPurge4Args {
    pub client_id: ClientId4,
}


