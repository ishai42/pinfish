use super::{SessionId4, SequenceId4, SlotId4, Count4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;


/// channel_attrs4
#[derive(UnpackFrom, PackTo, Debug)]
pub struct ChannelAttrs4 {
    pub header_pad_size: Count4,
    pub max_request_size: Count4,
    pub max_response_size: Count4,
    pub max_response_size_cached: Count4,
    pub max_operation: Count4,
    pub max_requests: Count4,
    pub rdma_ird: Option<u32>,
}


/// SEQUENCE
#[derive(PackTo, Debug)]
pub struct Sequence4Args {
    pub session_id: SessionId4,
    pub sequence_id: SequenceId4,
    pub slot_id: SlotId4,
    pub highest_slot_id: SlotId4,
    pub cache_this: bool,
}

/// SEQUENCE
#[derive(UnpackFrom, PackTo, Debug)]
pub struct Sequence4ResOk {
    pub session_id: SessionId4,
    pub sequence_id: SequenceId4,
    pub slot_id: SlotId4,
    pub highest_slot_id: SlotId4,
    pub target_highest_slot_id: SlotId4,
    pub status_flags: u32,
}
