/// NFS4 Operations
use crate::xdr::{self, VecPackTo};
use pinfish_macros::{PackTo, VecPackTo};

const NFS4_SESSIONID_SIZE: usize = 16;

pub type SessionId4 = [u8; NFS4_SESSIONID_SIZE];
pub type SequenceId4 = u32;
pub type SlotId4 = u32;
pub type ClientId4 = u64;
pub type Count4 = u32;
pub type Verifier4 = u64; // really opaque[8]

const EXCHGID4_FLAG_SUPP_MOVED_REFER: u32 = 0x00000001;
const EXCHGID4_FLAG_SUPP_MOVED_MIGR: u32 = 0x00000002;

const EXCHGID4_FLAG_BIND_PRINC_STATEID: u32 = 0x00000100;

const EXCHGID4_FLAG_USE_NON_PNFS: u32 = 0x00010000;
const EXCHGID4_FLAG_USE_PNFS_MDS: u32 = 0x00020000;
const EXCHGID4_FLAG_USE_PNFS_DS: u32 = 0x00040000;

const EXCHGID4_FLAG_MASK_PNFS: u32 = 0x00070000;

const EXCHGID4_FLAG_UPD_CONFIRMED_REC_A: u32 = 0x40000000;
const EXCHGID4_FLAG_CONFIRMED_R: u32 = 0x80000000;

/// The NfsTime4 gives the number of seconds and nano seconds since
/// midnight or zero hour January 1, 1970 Coordinated Universal Time
/// (UTC).
#[derive(PackTo, Debug)]
pub struct NfsTime4 {
    pub Seconds: i64,
    pub NanoSeconds: u32,
}

/// 18.35 EXCHANGE_ID4args
#[derive(PackTo, Debug)]
pub struct ExchangeId4Args {
    ClientOwner: ClientOwner4,
    Flags: u32,
    StateProtect: StateProtect4A,
    ClientImplId: Option<NfsImplId4>,
}

#[derive(PackTo, Debug)]
pub struct NfsImplId4 {
    Domain: String,
    Name: String,
    Date: NfsTime4,
}

#[derive(PackTo, Debug)]
pub struct ClientOwner4 {
    Verifier: Verifier4,
    OwnerId: Vec<u8>,
}

#[derive(PackTo, Debug)]
pub enum StateProtect4A {
    None,
    // TODO: MachCred
    // TODO: Ssv
}

#[derive(PackTo, VecPackTo, Debug)]
pub enum CallbackSecParams4 {
    AuthNone,
    // TODO : AuthSys
    // TODO : RpcSecGss
}

///
#[derive(PackTo, Debug)]
pub struct Sequence4Args {
    pub SessionId: SessionId4,
    pub SequenceId: SequenceId4,
    pub SlotId: SlotId4,
    pub HighestSlotId: SlotId4,
    pub CacheThis: bool,
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
#[derive(VecPackTo, PackTo, Debug)]
pub struct CreateSession4Args {
    pub ClientId: ClientId4,
    pub Sequence: SequenceId4,

    pub Flags: u32,

    pub ForeChanAttrs: ChannelAttrs4,
    pub BackChanAttrs: ChannelAttrs4,

    pub CbProgram: u32,
    pub SecParams: Vec<CallbackSecParams4>,
}

#[derive(PackTo, Debug)]
pub enum ArgOp4 {
    CreateSession(CreateSession4Args),
    Illegal
}

/// NFS4 COMPOUND args.
pub struct Compound {
    pub Tag: String,
    pub MinorVersion: u32,
    pub ArgArray: Vec<ArgOp4>
}

impl Compound {
    /// Create a new empty compound for version 4.1
    pub fn new() -> Compound {
        Compound {
            Tag: String::new(),
            MinorVersion: 1,
            ArgArray: Vec::new(),
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
            Seconds: (duration / nano) as i64,
            NanoSeconds: (duration % nano) as u32,
        }
    }
}

