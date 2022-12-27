use crate::rpc::RpcClient;
use tokio::net::TcpStream;
pub mod client;
pub mod ops;
pub const PROG_NFS: u32 = 100003;

pub const PROC_NULL: u32 = 0;
pub const PROC_COMPOUND: u32 = 1;

pub const NFS4_OK: u32 = 0;

struct Client {
    rpc: RpcClient,
}

impl Client {
    pub fn new(connection: TcpStream) -> Client {
        Client {
            rpc: RpcClient::new(connection),
        }
    }
}
