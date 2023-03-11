use crate::xdr;
use pinfish_macros::{PackTo, UnpackFrom};

/// TCP/UDP Port number for the RPC Port Mapper service and RPC bind
pub const PORT: u16 = 111;

pub const PMAP_VERS: u32 = 2;
pub const PMAP_PROG: u32 = 100000;

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Mapping {
    pub prog: u32,
    pub vers: u32,
    pub prot: u32,
    pub port: u32,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct RpcBind {
    pub prog: u32,
    pub vers: u32,
    pub net_id: String,
    pub addr: String,
    pub owner: String,
}

pub const IPPROTO_TCP: u32 = 6; /* protocol number for TCP/IP */
pub const IPPROTO_UDP: u32 = 17; /* protocol number for UDP/IP */

pub const PMAPPROC_NULL: u32 = 0;
pub const PMAPPROC_SET: u32 = 1;
pub const PMAPPROC_UNSET: u32 = 2;
pub const PMAPPROC_GETPORT: u32 = 3;
pub const PMAPPROC_DUMP: u32 = 4;
pub const PMAPPROC_CALLIT: u32 = 5;
