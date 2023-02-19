mod nfs4;
mod result;
mod rpc;
mod xdr;

use argh::FromArgs;
use std::error::Error;

#[derive(FromArgs)]
/// Test NFS client
struct Command {
    /// host name or IP address
    #[argh(option, short = 'h')]
    host: String,

    /// port, default is 2049
    #[argh(option, short = 'p', default = "2049")]
    port: u16,

    #[argh(subcommand)]
    cmd: Commands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Commands {
    Lookup(Lookup),
    Mkdir(Mkdir),
    Remove(Remove),
}

/// Lookup path and print the resulting FH
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "lookup")]
struct Lookup {
    #[argh(positional)]
    path: String,
}

/// Make a directory
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "mkdir")]
struct Mkdir {
    #[argh(positional)]
    path: String,
}

/// Delete a file or a directory
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "remove")]
struct Remove {
    #[argh(positional)]
    path: String,
}


fn split_last(path: &str) -> (&str, &str) {
    match path.rsplit_once('/') {
        None => ("", path),
        Some(x) => x,
    }
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

            println!("client_id = {:x}", client.client_id.get());

            client.create_session_call().await?;

            println!("session created");

//            client.send_reclaim_complete().await?;

//            println!("reclaim complete!");

            let (path, last) = match &cmd.cmd {
                Commands::Lookup(lookup) => (lookup.path.as_str(), ""),
                Commands::Mkdir(mkdir) => split_last(&mkdir.path),
                Commands::Remove(remove) => split_last(&remove.path),
            };

            let fh = client.resolve_path(&path).await?;
            println!("got fh {:?}", &fh);

            match &cmd.cmd {
                Commands::Lookup(_) => (),
                Commands::Mkdir(_) => {
                    let _fh = client.mkdir(&fh, last).await?;
                }
                Commands::Remove(_) => {
                    client.remove(&fh, last).await?;
                }
            };

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
