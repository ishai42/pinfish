use super::{NFS4_SESSION_ID_SIZE, NFS4_OTHER_SIZE};
use pinfish_macros::{PackTo, UnpackFrom, VecPackUnpack};
use crate::xdr::{self, VecPackUnpack};


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

/// The NfsTime4 gives the number of seconds and nano seconds since
/// midnight or zero hour January 1, 1970 Coordinated Universal Time
/// (UTC).
#[derive(PackTo, UnpackFrom, Debug)]
pub struct NfsTime4 {
    pub seconds: i64,
    pub nano_seconds: u32,
}

impl NfsTime4 {
    /// Get current system time as `NfsTime4`
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

#[derive(PackTo, VecPackUnpack, Debug)]
pub enum CallbackSecParams4 {
    AuthNone,
    // TODO : AuthSys
    // TODO : RpcSecGss
}

/// device numbers for block/char special devices
#[derive(PackTo, UnpackFrom, Debug)]
pub struct SpecData4 {
    major: u32,
    minor: u32,
}

#[derive(UnpackFrom, PackTo, Debug, Clone)]
pub struct ChangeInfo4 {
    atomic: bool,
    before: ChangeId4,
    after: ChangeId4,
}

#[derive(PackTo, Debug)]
pub struct OpenOwner4 {
    pub client_id: ClientId4,
    pub owner: bytes::Bytes,
}

#[derive(UnpackFrom, PackTo, Debug, Clone)]
pub struct StateId4 {
    sequence_id: u32,
    other: [u8; NFS4_OTHER_SIZE],
}

#[derive(UnpackFrom, PackTo, Debug, Clone)]
pub enum OpenDelegation4 {
    None,
    // Read(OpenReadDelegation4),
    // Write(OpenWriteDelegation4),
}
