/// NFS4 Operations
use crate::{
    xdr::{self, UnpackFrom, Unpacker, VecPackUnpack},
    result::Result
};
use pinfish_macros::{PackTo, UnpackFrom, VecPackUnpack};

const OP_EXCHANGE_ID: u32 = 42;
const OP_CREATE_SESSION: u32 = 43;
const OP_ILLEGAL: u32 = 10044;

const NFS4_session_id_SIZE: usize = 16;

pub type SessionId4 = [u8; NFS4_session_id_SIZE];
pub type SequenceId4 = u32;
pub type SlotId4 = u32;
pub type ClientId4 = u64;
pub type Count4 = u32;
pub type Verifier4 = u64; // really opaque[8]

pub const EXCHGID4_FLAG_SUPP_MOVED_REFER: u32 = 0x00000001;
pub const EXCHGID4_FLAG_SUPP_MOVED_MIGR: u32 = 0x00000002;

pub const EXCHGID4_FLAG_BIND_PRINC_STATEID: u32 = 0x00000100;

pub const EXCHGID4_FLAG_USE_NON_PNFS: u32 = 0x00010000;
pub const EXCHGID4_FLAG_USE_PNFS_MDS: u32 = 0x00020000;
pub const EXCHGID4_FLAG_USE_PNFS_DS: u32 = 0x00040000;

pub const EXCHGID4_FLAG_MASK_PNFS: u32 = 0x00070000;

pub const EXCHGID4_FLAG_UPD_CONFIRMED_REC_A: u32 = 0x40000000;
pub const EXCHGID4_FLAG_CONFIRMED_R: u32 = 0x80000000;

/// The NfsTime4 gives the number of seconds and nano seconds since
/// midnight or zero hour January 1, 1970 Coordinated Universal Time
/// (UTC).
#[derive(PackTo, UnpackFrom, Debug)]
pub struct NfsTime4 {
    pub seconds: i64,
    pub nano_seconds: u32,
}

/// 18.35 EXCHANGE_ID4args
#[derive(PackTo, UnpackFrom, Debug)]
pub struct ExchangeId4Args {
    pub client_owner: ClientOwner4,
    pub flags: u32,
    pub state_protect: StateProtect4A,
    pub client_impl_id: Option<NfsImplId4>,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct NfsImplId4 {
    pub domain: String,
    pub name: String,
    pub date: NfsTime4,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct ClientOwner4 {
    pub verifier: Verifier4,
    pub owner_id: Vec<u8>,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub enum StateProtect4A {
    None,
    // TODO: MachCred
    // TODO: Ssv
}

/// 18.35.2 EXCHANGE_ID4resok
#[derive(PackTo, UnpackFrom, Debug)]
pub struct ExchangeId4ResOk {
    pub client_id: ClientId4,
    pub sequence_id: SequenceId4,
    pub flags: u32,
    pub state_protect: StateProtect4R,
    pub server_owner: ServerOwner4,
    pub server_scope: Vec<u8>,
    pub server_impl_id: Option<NfsImplId4>,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct ServerOwner4 {
    pub minor_id: u64,
    pub major_id: Vec<u8>,
}

#[derive(PackTo, UnpackFrom, VecPackUnpack, Debug)]
pub enum StateProtect4R {
    None,
    // TODO: MachCred
    // TODO: Ssv
}

#[derive(PackTo, VecPackUnpack, Debug)]
pub enum CallbackSecParams4 {
    AuthNone,
    // TODO : AuthSys
    // TODO : RpcSecGss
}

///
#[derive(PackTo, Debug)]
pub struct Sequence4Args {
    pub session_id: SessionId4,
    pub sequence_id: SequenceId4,
    pub slot_id: SlotId4,
    pub highest_slot_id: SlotId4,
    pub cache_this: bool,
}

#[derive(PackTo, Debug)]
pub struct ChannelAttrs4 {
    pub HeaderPadSize: Count4,
    pub MaxRequestSize: Count4,
    pub MaxResponseSize: Count4,
    pub MaxResponseSizeCached: Count4,
    pub MaxOperation: Count4,
    pub MaxRequests: Count4,
    pub RdmaIrd: Option<u32>,
}

/// 18.36 -- CREATE_SESSION4args
#[derive(VecPackUnpack, PackTo, Debug)]
pub struct CreateSession4Args {
    pub client_id: ClientId4,
    pub sequence: SequenceId4,

    pub flags: u32,

    pub fore_chan_attrs: ChannelAttrs4,
    pub back_chan_attrs: ChannelAttrs4,

    pub cb_program: u32,
    pub sec_params: Vec<CallbackSecParams4>,
}

#[derive(PackTo, Debug, VecPackUnpack)]
pub enum ArgOp4 {
    #[xdr(OP_EXCHANGE_ID)]
    ExchangeId(ExchangeId4Args),
    #[xdr(OP_CREATE_SESSION)]
    CreateSession(CreateSession4Args),
    #[xdr(OP_ILLEGAL)]
    Illegal,
}

/// NFS4 COMPOUND args.
#[derive(PackTo, Debug)]
pub struct Compound {
    pub tag: String,
    pub minor_version: u32,
    pub arg_array: Vec<ArgOp4>,
}

impl Compound {
    /// Create a new empty compound for version 4.1
    pub fn new() -> Compound {
        Compound {
            tag: String::new(),
            minor_version: 1,
            arg_array: Vec::new(),
        }
    }
}

impl NfsTime4 {
    pub fn now() -> NfsTime4 {
        let now = std::time::SystemTime::now();
        let duration = now
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("system time before Unix epoch");

        let duration = duration.as_nanos();
        let nano = 1000 * 1000 * 1000;
        NfsTime4 {
            seconds: (duration / nano) as i64,
            nano_seconds: (duration % nano) as u32,
        }
    }
}

#[derive(UnpackFrom, Debug, VecPackUnpack)]
pub enum ResultOp4 {
    #[xdr(OP_EXCHANGE_ID)]
    ExchangeId(core::result::Result<ExchangeId4ResOk, u32>),
}

/// NFS4 COMPOUND result.
#[derive(UnpackFrom, Debug)]
pub struct CompoundResult {
    pub status: u32,
    pub tag: String,
    pub result_array: Vec<ResultOp4>,
}

impl<T: UnpackFrom<B>, B: Unpacker> UnpackFrom<B> for core::result::Result<T, u32> {
    fn unpack_from(buf: &mut B) -> Result<Self> {
        let n = u32::unpack_from(buf)?;
        match n {
            0 => Ok(Ok(T::unpack_from(buf)?)),
            _ => Ok(Err(n)),
        }
    }
}
