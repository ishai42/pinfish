use super::{ClientId4, SequenceId4, ChannelAttrs4, SessionId4, CallbackSecParams4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;

pub const CREATE_SESSION4_FLAG_PERSIST: u32 = 0x00000001;
pub const CREATE_SESSION4_FLAG_CONN_BACK_CHAN: u32 = 0x00000002;
pub const CREATE_SESSION4_FLAG_CONN_RDMA: u32 = 0x00000004;

/// 18.36 -- CREATE_SESSION4args
#[derive(PackTo, Debug)]
pub struct CreateSession4Args {
    pub client_id: ClientId4,
    pub sequence: SequenceId4,

    pub flags: u32,

    pub fore_chan_attrs: ChannelAttrs4,
    pub back_chan_attrs: ChannelAttrs4,

    pub cb_program: u32,
    pub sec_params: Vec<CallbackSecParams4>,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct CreateSession4ResOk {
    pub session_id: SessionId4,
    pub sequence: SequenceId4,

    pub flags: u32,

    pub fore_chan_attrs: ChannelAttrs4,
    pub back_chan_attrs: ChannelAttrs4,
}
