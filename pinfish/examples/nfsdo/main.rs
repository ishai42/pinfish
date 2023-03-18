use pinfish::{
    nfs4::{
        self,
        ops::{OPEN4_SHARE_ACCESS_READ, OPEN4_SHARE_DENY_NONE},
    },
    result,
};

use argp::FromArgs;
use std::error::Error;
use std::io::{self, Write};

#[derive(FromArgs)]
/// Test NFS client
struct Command {
    /// host name or IP address
    #[argp(option, short = 'h')]
    host: String,

    /// port, default is 2049
    #[argp(option, short = 'p', default = "2049")]
    port: u16,

    #[argp(subcommand)]
    cmd: Commands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argp(subcommand)]
enum Commands {
    Lookup(Lookup),
    Mkdir(Mkdir),
    Remove(Remove),
    ReadDir(ReadDir),
    Read(Read),
}

/// Lookup path and print the resulting FH
#[derive(FromArgs, PartialEq, Debug)]
#[argp(subcommand, name = "lookup")]
struct Lookup {
    #[argp(positional)]
    path: String,
}

/// Make a directory
#[derive(FromArgs, PartialEq, Debug)]
#[argp(subcommand, name = "mkdir")]
struct Mkdir {
    #[argp(positional)]
    path: String,
}

/// Delete a file or a directory
#[derive(FromArgs, PartialEq, Debug)]
#[argp(subcommand, name = "remove")]
struct Remove {
    #[argp(positional)]
    path: String,
}

/// Delete a file or a directory
#[derive(FromArgs, PartialEq, Debug)]
#[argp(subcommand, name = "ls")]
struct ReadDir {
    #[argp(positional)]
    path: String,
}

/// Read a file and send output to stdout
#[derive(FromArgs, PartialEq, Debug)]
#[argp(subcommand, name = "read")]
struct Read {
    #[argp(positional)]
    path: String,
}

fn split_last(path: &str) -> (&str, &str) {
    match path.rsplit_once('/') {
        None => ("", path),
        Some(x) => x,
    }
}

fn ls_print_entry(entry: &nfs4::ops::Entry4) {
    let obj_type = entry.attrs.obj_type.unwrap();
    let c1 = match obj_type {
        nfs4::attr::NfsType4::Reg => '_',
        nfs4::attr::NfsType4::Dir => 'd',
        nfs4::attr::NfsType4::Blk => 'b',
        nfs4::attr::NfsType4::Chr => 'c',
        nfs4::attr::NfsType4::Lnk => 'l',
        nfs4::attr::NfsType4::Sock => 's',
        _ => '?',
    };
    let mode = entry.attrs.mode.unwrap();
    let c2 = if (mode & 256) != 0 { 'r' } else { '-' };
    let c3 = if (mode & 128) != 0 { 'w' } else { '-' };
    let c4 = if (mode & 64) != 0 { 'x' } else { '-' };
    let c5 = if (mode & 32) != 0 { 'r' } else { '-' };
    let c6 = if (mode & 16) != 0 { 'w' } else { '-' };
    let c7 = if (mode & 8) != 0 { 'x' } else { '-' };
    let c8 = if (mode & 4) != 0 { 'r' } else { '-' };
    let c9 = if (mode & 2) != 0 { 'w' } else { '-' };
    let c10 = if (mode & 1) != 0 { 'x' } else { '-' };

    println!(
        "{}{}{}{}{}{}{}{}{}{} {}",
        c1, c2, c3, c4, c5, c6, c7, c8, c9, c10, entry.name
    );
}

async fn ls(client: &mut nfs4::client::NfsClient, fh: &nfs4::ops::NfsFh4) -> result::Result<()> {
    let mut eof = false;
    let mut cookie = 0;
    let mut cookie_verf = 0;

    while !eof {
        let result = client.readdir(fh, cookie, cookie_verf).await?;
        cookie_verf = result.cookie_verf;
        eof = result.reply.eof;
        for entry in result.reply.iter() {
            cookie = entry.cookie;
            ls_print_entry(&entry);
        }
    }

    Ok(())
}

async fn read(client: &mut nfs4::client::NfsClient, fh: &nfs4::ops::NfsFh4) -> result::Result<()> {
    let open_result = client
        .open_by_id(fh, OPEN4_SHARE_ACCESS_READ, OPEN4_SHARE_DENY_NONE)
        .await?;

    let mut eof = false;
    let mut offset = 0;
    const CHUNK: u32 = 1024 * 1024;
    while !eof {
        let result = client
            .read(fh, &open_result.state_id, offset, CHUNK)
            .await?;
        eof = result.eof;
        io::stdout().write_all(&result.data)?;
        offset += result.data.len() as u64;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cmd: Command = argp::from_env();
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

            match client.send_reclaim_complete().await {
                Ok(_) => (),
                Err(code) => {
                    if code.get() != result::NFS4ERR_COMPLETE_ALREADY {
                        Err(code)?;
                    }
                }
            }

            println!("reclaim complete!");

            let (path, last) = match &cmd.cmd {
                Commands::Lookup(lookup) => (lookup.path.as_str(), ""),
                Commands::Mkdir(mkdir) => split_last(&mkdir.path),
                Commands::Remove(remove) => split_last(&remove.path),
                Commands::ReadDir(readdir) => (readdir.path.as_str(), ""),
                Commands::Read(read) => (read.path.as_str(), ""),
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
                Commands::ReadDir(_) => ls(&mut client, &fh).await?,
                Commands::Read(_) => read(&mut client, &fh).await?,
            };

            Ok(())
        })
}
