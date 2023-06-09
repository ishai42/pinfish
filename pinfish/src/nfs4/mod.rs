//! Definitions for encoding/decoding NFSv4.1 calls and replies.
pub mod client;
pub mod ops;
pub mod sequence;
pub const PROG_NFS: u32 = 100003;
pub mod attr;

pub const PROC_NULL: u32 = 0;
pub const PROC_COMPOUND: u32 = 1;

pub const NFS4_OK: u32 = 0;
