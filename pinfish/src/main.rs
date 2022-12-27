mod nfs4;
mod result;
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
            let mut client = nfs4::client::NfsClient::new(&host_string);
            client.connect().await?;
            println!("Connected");
            client.null_call().await?;
            println!("\n\ncompleted null call");

            client.exchange_id_call().await?;

            println!("client_id = {:x}", client.client_id);

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
