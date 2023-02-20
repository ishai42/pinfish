/// NFS4 Operations
use crate::{
    result::Result,
    xdr::{self, UnpackFrom, Unpacker, VecPackUnpack},
};
use pinfish_macros::{PackTo, UnpackFrom, VecPackUnpack};

pub use super::attr::{Bitmap4, FileAttributes, NfsType4};

const OP_CREATE: u32 = 6;
const OP_GETFH: u32 = 10;
const OP_LOOKUP: u32 = 15;
const OP_PUTFH: u32 = 22;
const OP_READDIR: u32 = 26;
const OP_REMOVE: u32 = 28;
const OP_PUTROOTFH: u32 = 24;
const OP_EXCHANGE_ID: u32 = 42;
const OP_CREATE_SESSION: u32 = 43;
const OP_SEQUENCE: u32 = 53;
const OP_RECLAIM_COMPLETE: u32 = 58;
const OP_ILLEGAL: u32 = 10044;

const NFS4_SESSION_ID_SIZE: usize = 16;
// const NFS4_VERIFIER_SIZE: usize = 8;
const NFS4_OTHER_SIZE: usize = 12;

pub type SessionId4 = [u8; NFS4_SESSION_ID_SIZE];
pub type SequenceId4 = u32;
pub type SlotId4 = u32;
pub type ClientId4 = u64;
pub type Count4 = u32;
pub type Verifier4 = u64; // really opaque[8]
pub type NfsFh4 = Vec<u8>; // should be opaque<NFS4_FHSIZE>
pub type Component4 = String;
pub type ChangeId4 = u64;
pub type Cookie4 = u64;

pub const EXCHGID4_FLAG_SUPP_MOVED_REFER: u32 = 0x00000001;
pub const EXCHGID4_FLAG_SUPP_MOVED_MIGR: u32 = 0x00000002;

pub const EXCHGID4_FLAG_BIND_PRINC_STATEID: u32 = 0x00000100;

pub const EXCHGID4_FLAG_USE_NON_PNFS: u32 = 0x00010000;
pub const EXCHGID4_FLAG_USE_PNFS_MDS: u32 = 0x00020000;
pub const EXCHGID4_FLAG_USE_PNFS_DS: u32 = 0x00040000;

pub const EXCHGID4_FLAG_MASK_PNFS: u32 = 0x00070000;

pub const EXCHGID4_FLAG_UPD_CONFIRMED_REC_A: u32 = 0x40000000;
pub const EXCHGID4_FLAG_CONFIRMED_R: u32 = 0x80000000;

pub const OPEN4_SHARE_ACCESS_READ: u32 = 0x00000001;
pub const OPEN4_SHARE_ACCESS_WRITE: u32 = 0x00000002;
pub const OPEN4_SHARE_ACCESS_BOTH: u32 = 0x00000003;

pub const OPEN4_SHARE_DENY_NONE: u32 = 0x00000000;
pub const OPEN4_SHARE_DENY_READ: u32 = 0x00000001;
pub const OPEN4_SHARE_DENY_WRITE: u32 = 0x00000002;
pub const OPEN4_SHARE_DENY_BOTH: u32 = 0x00000003;

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

/// LOOKUP
#[derive(PackTo, Debug)]
pub struct Lookup4Args {
    pub objname: Component4,
}

/// PUTFH
#[derive(PackTo, Debug)]
pub struct PutFh4Args {
    pub object: NfsFh4,
}

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

pub const CREATE_SESSION4_FLAG_PERSIST: u32 = 0x00000001;
pub const CREATE_SESSION4_FLAG_CONN_BACK_CHAN: u32 = 0x00000002;
pub const CREATE_SESSION4_FLAG_CONN_RDMA: u32 = 0x00000004;

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

#[derive(UnpackFrom, PackTo, Debug)]
pub struct CreateSession4ResOk {
    pub session_id: SessionId4,
    pub sequence: SequenceId4,

    pub flags: u32,

    pub fore_chan_attrs: ChannelAttrs4,
    pub back_chan_attrs: ChannelAttrs4,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct ReclaimComplete4Args {
    pub one_fs: bool,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct GetFh4ResOk {
    pub object: NfsFh4,
}

/// device numbers for block/char special devices
#[derive(PackTo, Debug)]
pub struct SpecData4 {
    major: u32,
    minor: u32,
}

#[derive(PackTo, Debug)]
pub enum CreateType4 {
    /// Symbolic link
    #[xdr(5)]
    Link(String),
    /// Block device
    #[xdr(3)]
    Block(SpecData4),
    /// Char device
    #[xdr(4)]
    Char(SpecData4),
    /// Socket
    #[xdr(6)]
    Socket,
    /// FIFO
    #[xdr(7)]
    Fifo,
    /// Directory
    #[xdr(2)]
    Directory,
}

#[derive(PackTo, Debug)]
pub struct Create4Args {
    pub objtype: CreateType4,
    pub component: String,
    pub attributes: FileAttributes,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct ChangeInfo4 {
    atomic: bool,
    before: ChangeId4,
    after: ChangeId4,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct Create4ResOk {
    change_info: ChangeInfo4,
    attr_set: Bitmap4,
}

#[derive(PackTo, Debug)]
pub struct Remove4Args {
    pub target: Component4,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct Remove4ResOk {
    pub change_info: ChangeInfo4,
}

#[derive(PackTo, Debug)]
pub struct ReadDir4Args {
    pub cookie: Cookie4,
    pub verifier: Verifier4,
    pub dir_count: Count4,
    pub max_count: Count4,
    pub attr_request: Bitmap4,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct Entry4 {
    pub cookie: Cookie4,
    pub name: Component4,
    pub attrs: FileAttributes,
    pub next_entry: Option<Box<Entry4>>,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct DirList4 {
    pub entries: Option<Entry4>,
    pub eof: bool,
}

/// Iterator for directory entries
pub struct Entry4Iter<'a> {
    next: Option<&'a Entry4>,
}

impl<'a> Iterator for Entry4Iter<'a> {
    type Item = &'a Entry4;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.next {
            self.next = entry.next_entry.as_ref().map(|e| &**e);
            Some(entry)
        } else {
            None
        }
    }
}

impl DirList4 {
    /// Iterate over directory entries
    pub fn iter(&self) -> Entry4Iter<'_> {
        Entry4Iter {
            next: self.entries.as_ref(),
        }
    }
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct ReadDir4ResOk {
    pub cookie_verf: Verifier4,
    pub reply: DirList4,
}

#[derive(PackTo, Debug)]
pub struct OpenOwner4 {
    pub client_id: ClientId4,
    pub owner: bytes::Bytes,
}

#[derive(PackTo, Debug)]
pub enum OpenFlag4 {
    NoCreate,
    Create(CreateHow4)
}

#[derive(PackTo, Debug)]
pub enum CreateHow4 {
    Unchecked(FileAttributes),
    Guarded(FileAttributes),
    Exclusive(Verifier4),
}

#[derive(PackTo, Debug)]
pub enum OpenClaim4 {
    Null(String),
//    Previous(OpenDelegationType4),
//    DelegateCur(OpenClaimDelegateCur4),
//    DelegatePrev(String),
}

#[derive(PackTo, Debug)]
pub struct Open4Args {
    pub sequence_id: SequenceId4,
    pub share_access: u32,
    pub share_deny: u32,
    pub owner: OpenOwner4,
    pub how: OpenFlag4,
    pub claim: OpenClaim4,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct StateId4 {
    sequence_id: u32,
    other: [u8; NFS4_OTHER_SIZE],
}

#[derive(UnpackFrom, PackTo, Debug)]
pub enum OpenDelegation4 {
    None,
    // Read(OpenReadDelegation4),
    // Write(OpenWriteDelegation4),
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct Open4ResOk {
    state_id: StateId4,
    change_info: ChangeInfo4,
    result_flags: u32,
    attr_set: Bitmap4,
    delegation: OpenDelegation4,
}

// --------------

#[derive(PackTo, Debug, VecPackUnpack)]
pub enum ArgOp4 {
    #[xdr(OP_CREATE)] // 6
    Create(Create4Args),

    #[xdr(OP_GETFH)] // 10
    GetFh,

    #[xdr(OP_LOOKUP)] // 15
    Lookup(Lookup4Args),

    #[xdr(OP_PUTFH)] // 22
    PutFh(PutFh4Args),

    #[xdr(OP_PUTROOTFH)] // 24
    PutRootFh,

    #[xdr(OP_READDIR)] // 26
    ReadDir(ReadDir4Args),

    #[xdr(OP_REMOVE)] // 28
    Remove(Remove4Args),

    #[xdr(OP_EXCHANGE_ID)] // 42
    ExchangeId(ExchangeId4Args),

    #[xdr(OP_CREATE_SESSION)] // 43
    CreateSession(CreateSession4Args),

    #[xdr(OP_SEQUENCE)] // 53
    Sequence(Sequence4Args),

    #[xdr(OP_RECLAIM_COMPLETE)] // 58
    ReclaimComplete(ReclaimComplete4Args),

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
    #[xdr(OP_CREATE)] // 6
    Create(core::result::Result<Create4ResOk, u32>),

    #[xdr(OP_GETFH)] // 10
    GetFh(core::result::Result<GetFh4ResOk, u32>),

    #[xdr(OP_LOOKUP)] // 15
    Lookup(core::result::Result<(), u32>),

    #[xdr(OP_PUTFH)] // 22
    PutFh(core::result::Result<(), u32>),

    #[xdr(OP_PUTROOTFH)] // 24
    PutRootFh(core::result::Result<(), u32>),

    #[xdr(OP_READDIR)] // 26
    ReadDir(core::result::Result<ReadDir4ResOk, u32>),

    #[xdr(OP_REMOVE)] // 28
    Remove(core::result::Result<Remove4ResOk, u32>),

    #[xdr(OP_EXCHANGE_ID)] // 42
    ExchangeId(core::result::Result<ExchangeId4ResOk, u32>),

    #[xdr(OP_CREATE_SESSION)] // 43
    CreateSession(core::result::Result<CreateSession4ResOk, u32>),

    #[xdr(OP_SEQUENCE)] // 53
    Sequence(core::result::Result<Sequence4ResOk, u32>),

    #[xdr(OP_RECLAIM_COMPLETE)] // 58
    ReclaimComplete(core::result::Result<(), u32>),

    #[xdr(OP_ILLEGAL)]
    Illegal(core::result::Result<(), u32>),
}

/// NFS4 COMPOUND result.
#[derive(UnpackFrom, Debug)]
pub struct CompoundResult {
    pub status: u32,
    pub tag: String,
    pub result_array: Vec<ResultOp4>,
}

impl<T: core::fmt::Debug + UnpackFrom<B>, B: Unpacker> UnpackFrom<B>
    for core::result::Result<T, u32>
{
    fn unpack_from(buf: &mut B) -> Result<Self> {
        let n = u32::unpack_from(buf)?;
        match n {
            0 => Ok(Ok(T::unpack_from(buf)?)),
            _ => Ok(Err(n)),
        }
    }
}
