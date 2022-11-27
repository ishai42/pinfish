mod nfs4;
mod rpc;
mod xdr;

use crate::xdr::Packer as XdrPacker;
use argh::FromArgs;
use bytes::Buf;
use rpc::Packer;
use std::borrow::BorrowMut;
use std::error::Error;
use tokio::net::TcpStream;

#[derive(FromArgs)]
/// Test NFS client
struct Command {
    /// host name or IP address
    #[argh(option, short = 'h')]
    host: String,

    /// port, default is 2049
    #[argh(option, short = 'p', default = "2049")]
    port: u16,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cmd: Command = argh::from_env();
    let host_string = std::format!("{}:{}", cmd.host, cmd.port);

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let connection = TcpStream::connect(host_string).await?;
            println!("Connected");
            let mut client = rpc::RpcClient::new(connection);
            let mut buf = bytes::BytesMut::new();
            let xid = client.next_xid();
            let header = rpc::CallHeader {
                prog: nfs4::PROG_NFS,
                vers: 4,
                proc: nfs4::PROC_NULL,
                cred: rpc::OpaqueAuth::new_sys(
                    1,
                    bytes::Bytes::from_static(b"blah"),
                    0,
                    0,
                    Vec::new(),
                ),
                verf: rpc::OpaqueAuth::new_none(),
            };

            buf.pack_uint(0); // placeholder for frag
            buf.pack_uint(xid);
            buf.pack_call_header(&header);
            let frag_size = (buf.remaining() - 4) as u32;
            let frag_size = frag_size | 0x80000000;
            {
                let borrow: &mut [u8] = buf.borrow_mut();
                (&mut borrow[0..4]).pack_uint(frag_size);
            }

            let _response_buf = client.call(buf.freeze(), xid).await?;

            println!("got response");

            Ok(())
        })
}

/*
#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
*/
