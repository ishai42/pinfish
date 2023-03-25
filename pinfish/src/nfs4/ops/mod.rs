/// NFS4 Operations
use crate::{
    result::Result,
    xdr::{self, UnpackFrom, Unpacker, VecPackUnpack},
};
use pinfish_macros::{PackTo, UnpackFrom, VecPackUnpack};

pub use super::attr::{Bitmap4, FileAttributes, NfsType4};

pub_use!(types);

const OP_CREATE: u32 = 6;
const OP_GETFH: u32 = 10;
const OP_LOOKUP: u32 = 15;
const OP_OPEN: u32 = 18;
const OP_PUTFH: u32 = 22;
const OP_READ: u32 = 25;
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

// --------------

#[derive(PackTo, Debug, VecPackUnpack)]
pub enum ArgOp4 {
    #[xdr(OP_CREATE)] // 6
    Create(Create4Args),

    #[xdr(OP_GETFH)] // 10
    GetFh,

    #[xdr(OP_LOOKUP)] // 15
    Lookup(Lookup4Args),

    #[xdr(OP_OPEN)] // 18
    Open(Open4Args),

    #[xdr(OP_PUTFH)] // 22
    PutFh(PutFh4Args),

    #[xdr(OP_PUTROOTFH)] // 24
    PutRootFh,

    #[xdr(OP_READ)] // 25
    Read(Read4Args),

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

#[derive(UnpackFrom, Debug, VecPackUnpack)]
pub enum ResultOp4 {
    #[xdr(OP_CREATE)] // 6
    Create(core::result::Result<Create4ResOk, u32>),

    #[xdr(OP_GETFH)] // 10
    GetFh(core::result::Result<GetFh4ResOk, u32>),

    #[xdr(OP_LOOKUP)] // 15
    Lookup(core::result::Result<(), u32>),

    #[xdr(OP_OPEN)] // 18
    Open(core::result::Result<Open4ResOk, u32>),

    #[xdr(OP_PUTFH)] // 22
    PutFh(core::result::Result<(), u32>),

    #[xdr(OP_PUTROOTFH)] // 24
    PutRootFh(core::result::Result<(), u32>),

    #[xdr(OP_READ)] // 25
    Read(core::result::Result<Read4ResOk, u32>),

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

pub_use!(
    exchange_id,
    lookup,
    sequence,
    create_session,
    create,
    remove,
    putfh
);
pub_use!(reclaim_complete, getfh, readdir, open, read);
