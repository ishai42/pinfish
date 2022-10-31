use crate::xdr::{self, Unpacker};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpStream};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;


const MAX_PACKET_SIZE: u32 = 1024*1024;

// RFC5531  RPC v2

const LAST_FRAGMENT: u32 = 0x80000000;
const CALL: u32 = 0;
const REPLY: u32 = 1;

const AUTH_NONE: u32 = 0;
const AUTH_SYS: u32 = 1;

pub struct AuthSys {
    pub stamp: u32,
    pub machine_name: Bytes,
    pub uid: u32,
    pub gid: u32,
    pub gids: Vec<u32>,
}

/// RFC5531 opaque_auth
pub enum OpaqueAuth {
    None,
    Sys(AuthSys),
}

/// Corresponds to RFC5531 call_body.
pub struct CallHeader {
    // rpcvers is hardcoded 2
    pub prog: u32,
    pub vers: u32,
    pub proc: u32,
    pub cred: OpaqueAuth,
    pub verf: OpaqueAuth,
}

/// Reads an RPC packet from `stream`, potentially comprised of
/// multiple fragments into `buf`.  Allows at most
/// `max_size` bytes.
///
/// Panics if `buf` capacity is less than `max_size`
pub async fn read_packet<S: AsyncReadExt + Unpin, B: BufMut>(
    stream: &mut S,
    buf: &mut B,
    max_size: u32,
) -> io::Result<()> {
    assert!(buf.remaining_mut() >= max_size as usize);

    let mut record_mark_buf: [u8; 4] = [0; 4];
    let mut read_last = false;
    let mut len = 0;
    while !read_last {
        stream.read_exact(&mut record_mark_buf).await?;
        let record_mark = record_mark_buf.as_ref().unpack_uint();
        if (record_mark & LAST_FRAGMENT) != 0 {
            read_last = true;
        }

        let fragment_size = record_mark & !LAST_FRAGMENT;
        if fragment_size > max_size
            || fragment_size.wrapping_add(len) < fragment_size
            || fragment_size + len > max_size
        {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        let mut lim_buf = buf.limit(fragment_size as usize);
        while lim_buf.remaining_mut() > 0 {
            let read = stream.read_buf(&mut lim_buf).await?;
            if read == 0 {
                return Err(io::Error::from(io::ErrorKind::ConnectionAborted));
            }
        }

        len += fragment_size;
    }

    Ok(())
}

/// Trait for packing RPC header
pub trait Packer {
    fn pack_call_header(&mut self, header: &CallHeader);
    fn pack_auth(&mut self, auth: &OpaqueAuth);
    fn pack_auth_sys(&mut self, auth: &AuthSys);
}

impl<T: xdr::Packer> Packer for T {
    fn pack_call_header(&mut self, header: &CallHeader) {
        self.pack_uint(CALL);
        self.pack_uint(2); // rpcvers, must be 2 per RFC5531
        self.pack_uint(header.prog);
        self.pack_uint(header.vers);
        self.pack_uint(header.proc);
        self.pack_auth(&header.cred);
        self.pack_auth(&header.verf);
    }

    fn pack_auth(&mut self, auth: &OpaqueAuth) {
        match auth {
            OpaqueAuth::None => {
                self.pack_uint(AUTH_NONE);
                self.pack_uint(0)
            }
            OpaqueAuth::Sys(auth_sys) => {
                self.pack_uint(AUTH_SYS);
                self.pack_auth_sys(auth_sys)
            }
        }
    }

    fn pack_auth_sys(&mut self, auth: &AuthSys) {
        self.pack_uint((auth.gids.len() + auth.machine_name.len()) as u32 + 20);
        self.pack_uint(auth.stamp);
        self.pack_opaque(&*auth.machine_name);
        self.pack_uint(auth.uid);
        self.pack_uint(auth.gid);
        self.pack_array(&auth.gids, |packer: &mut Self, item| {
            packer.pack_uint(*item)
        });
    }
}

impl OpaqueAuth {
    pub fn new_none() -> OpaqueAuth {
        OpaqueAuth::None
    }

    pub fn new_sys(
        stamp: u32,
        machine_name: Bytes,
        uid: u32,
        gid: u32,
        gids: Vec<u32>,
    ) -> OpaqueAuth {
        OpaqueAuth::Sys(AuthSys {
            stamp,
            machine_name,
            uid,
            gid,
            gids,
        })
    }
}


struct RpcClientReceiver {
    connection: ReadHalf<TcpStream>,
    pending: Arc<Mutex<BTreeMap<u32, oneshot::Sender<Bytes>>>>,
    max_size: u32,
}

impl RpcClientReceiver {
    pub async fn run(&mut self) -> io::Result<()> {
        loop {
            let mut buf = BytesMut::new();
            read_packet(&mut self.connection, &mut buf, self.max_size).await?;
            let mut buf = buf.freeze();
            if buf.remaining() < 8 {
                println!("bad packet -- too short");
                continue;
            }

            let xid = buf.unpack_uint();
            let msg_type = buf.unpack_uint();
            match msg_type {
                CALL => println!("CB not implemented yet"),
                REPLY => {
                    let mut pending = self.pending.lock().unwrap();
                    let tx = pending.remove(&xid);
                    drop(pending);
                    match tx {
                        None => println!("unmatched xid {}", xid),
                        Some(tx) => { tx.send(buf); }
                    }
                }
                _ => println!("corrupt packet msg_type={}", msg_type)
            }

        }
    }
}


pub struct RpcClient {
    xid: u32,
    connection: tokio::sync::Mutex<WriteHalf<TcpStream>>,
    pending: Arc<Mutex<BTreeMap<u32, oneshot::Sender<Bytes>>>>,
}


impl RpcClient {
    pub fn new(connection: TcpStream) -> RpcClient {
        let (read, write) = tokio::io::split(connection);
        let pending = Arc::new(Mutex::new(BTreeMap::new()));

        let mut reader = RpcClientReceiver{
            connection: read,
            pending: pending.clone(),
            max_size: MAX_PACKET_SIZE,
        };

        tokio::spawn(async move {
            reader.run().await;
        });

        RpcClient{
            xid: 1,
            connection: tokio::sync::Mutex::new(write),
            pending,
        }
    }

    pub fn next_xid(&mut self) -> u32 {
        self.xid += 1;
        self.xid
    }

//    pub async fn read_packet<BB: BufMut>(&mut self, buf: &mut BB, max_size: u32) -> io::Result<()> {
//        read_packet(&mut self.connection, buf, max_size).await
    //    }


    pub async fn call(&mut self, buf : impl Buf, xid: u32) -> io::Result<Bytes> {
        let (tx, rx) = oneshot::channel();
        let mut pending = self.pending.lock().unwrap();
        pending.insert(xid, tx);
        drop(pending);

        self.send(buf).await?;

        rx.await.map_err(|_| io::ErrorKind::Other.into())
    }

    async fn send(&mut self, mut buf : impl Buf) -> io::Result<()> {
        let mut connection = self.connection.lock().await;
        while buf.has_remaining() {
            connection.write_buf(&mut buf).await?;
        }

        Ok(())
    }
}

