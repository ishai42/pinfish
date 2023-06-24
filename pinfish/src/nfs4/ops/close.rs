use super::{SequenceId4, StateId4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;



#[derive(PackTo, UnpackFrom, Debug)]
pub struct Close4Args {
    /// The "seqid" field of the request is not used in NFSv4.1
    pub seqid: SequenceId4,
    pub state_id: StateId4,
}

#[derive(PackTo, UnpackFrom, Debug, Clone)]
pub struct Close4ResOk {
    pub state_id: StateId4,
}
