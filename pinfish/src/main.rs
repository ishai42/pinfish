mod nfs4;
mod rpc;
mod xdr;

use crate::xdr::Packer as XdrPacker;
use crate::xdr::{PackTo, UnpackFrom};
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

            println!("got response for NULL");

            let mut buf = bytes::BytesMut::new();
            let xid = client.next_xid();
            let header = rpc::CallHeader {
                prog: nfs4::PROG_NFS,
                vers: 4,
                proc: nfs4::PROC_COMPOUND,
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

            let mut compound = nfs4::ops::Compound::new();
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::ExchangeId(nfs4::ops::ExchangeId4Args {
                    client_owner: nfs4::ops::ClientOwner4 {
                        verifier: 0,
                        owner_id: Vec::from(*b"owner/id/string"),
                    },
                    flags: nfs4::ops::EXCHGID4_FLAG_USE_PNFS_MDS
                        | nfs4::ops::EXCHGID4_FLAG_SUPP_MOVED_REFER,
                    state_protect: nfs4::ops::StateProtect4A::None,
                    client_impl_id: None,
                }));

            compound.pack_to(&mut buf);
            let frag_size = (buf.remaining() - 4) as u32;
            let frag_size = frag_size | 0x80000000;
            {
                let borrow: &mut [u8] = buf.borrow_mut();
                (&mut borrow[0..4]).pack_uint(frag_size);
            }

            let mut response_buf = client.call(buf.freeze(), xid).await?;
            let header = rpc::ReplyHeader::unpack_from(&mut response_buf);
            let resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf);

            println!("got response for EXCHANGE_ID {:?}//{:?}", header, resp);

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
