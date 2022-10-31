use crate::rpc::RpcClient;
use tokio::net::{TcpStream};

pub const PROG_NFS: u32 = 100003;

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
