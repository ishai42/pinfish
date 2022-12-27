use crate::{
    result::{ErrorCode, Result, INVALID_DATA, RPC_PROG_UNAVAIL, RPC_PROG_MISMATCH, RPC_PROC_UNAVAIL, RPC_GARBAGE_ARGS, RPC_SYSTEM_ERR, RPC_REJECTED_MISMATCH, RPC_REJECTED_AUTH_ERROR},
    xdr::{self, UnpackFrom, Unpacker as _},
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use pinfish_macros::{PackTo, UnpackFrom};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, atomic::{self, AtomicU32}};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::oneshot;

const MAX_PACKET_SIZE: u32 = 1024 * 1024;



// RFC5531  RPC v2

const LAST_FRAGMENT: u32 = 0x80000000;
const CALL: u32 = 0;
const REPLY: u32 = 1;

const AUTH_NONE: u32 = 0;
const AUTH_SYS: u32 = 1;

const MSG_ACCEPTED: u32 = 0;
const MSG_DENIED: u32 = 1;

#[derive(Debug)]
pub struct AuthSys {
    pub stamp: u32,
    pub machine_name: Bytes,
    pub uid: u32,
    pub gid: u32,
    pub gids: Vec<u32>,
}

/// RFC5531 opaque_auth
#[derive(Debug)]
pub enum OpaqueAuth {
    None,
    Sys(AuthSys),
}

impl<B: Packer> xdr::PackTo<B> for OpaqueAuth {
    fn pack_to(&self, buf: &mut B) {
        buf.pack_auth(self);
    }
}

impl<B: Unpacker> xdr::UnpackFrom<B> for OpaqueAuth {
    fn unpack_from(buf: &mut B) -> Result<Self> {
        buf.unpack_auth()
    }
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

impl<B: Packer> xdr::PackTo<B> for CallHeader {
    #[inline]
    fn pack_to(&self, buf: &mut B) {
        buf.pack_call_header(self);
    }
}


/// Corresponds to RFC5531 reply_body
#[derive(UnpackFrom, PackTo, Debug)]
pub enum ReplyHeader {
    Accepted(AcceptedReply),
    Denied(RejectedReply),
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct AcceptedReply {
    pub verf: OpaqueAuth,
    pub stat: AcceptedReplyStat,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub enum AcceptedReplyStat {
    Success,
    ProgUnavail,
    ProgMismatch(MismatchInfo),
    ProcUnavail,
    GarbageArgs,
    SystemErr,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct MismatchInfo {
    low: u32,
    high: u32,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub enum AuthStat {
    Ok,
    BadCred,
    RejectedCred,
    BadVerf,
    RejectedVerf,
    TooWeak,
    InvalidResp,
    Failed,
    KerbGeneric,
    TimeExpire,
    TktFile,
    Decode,
    NetAddr,
    CredProblem,
    CtxProblem,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub enum RejectedReply {
    RpcMismatch(MismatchInfo),
    AuthError(AuthStat),
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
) -> Result<()> {
    assert!(buf.remaining_mut() >= max_size as usize);

    let mut record_mark_buf: [u8; 4] = [0; 4];
    let mut read_last = false;
    let mut len = 0;
    while !read_last {
        stream.read_exact(&mut record_mark_buf).await?;
        let record_mark = record_mark_buf.as_ref().unpack_uint()?;
        if (record_mark & LAST_FRAGMENT) != 0 {
            read_last = true;
        }

        let fragment_size = record_mark & !LAST_FRAGMENT;
        if fragment_size > max_size
            || fragment_size.wrapping_add(len) < fragment_size
            || fragment_size + len > max_size
        {
            return Err(ErrorCode::new(INVALID_DATA));
        }

        let mut lim_buf = buf.limit(fragment_size as usize);
        while lim_buf.remaining_mut() > 0 {
            let read = stream.read_buf(&mut lim_buf).await?;
            if read == 0 {
                return Err(ErrorCode::new(INVALID_DATA));
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

/// Trait for unpacking RPC header
pub trait Unpacker {
    fn unpack_reply_header(&mut self) -> Result<ReplyHeader>;
    fn unpack_auth(&mut self) -> Result<OpaqueAuth>;
    fn unpack_auth_sys(&mut self) -> Result<AuthSys>;
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

impl<T: xdr::Unpacker> Unpacker for T {
    fn unpack_reply_header(&mut self) -> Result<ReplyHeader> {
        let reply_stat = self.unpack_uint()?;
        match reply_stat {
            MSG_ACCEPTED => Ok(ReplyHeader::Accepted(AcceptedReply::unpack_from(self)?)),
            MSG_DENIED => Ok(ReplyHeader::Denied(RejectedReply::unpack_from(self)?)),
            _ => Err(INVALID_DATA.into()),
        }
    }

    fn unpack_auth(&mut self) -> Result<OpaqueAuth> {
        let n = self.unpack_uint()?;
        match n {
            AUTH_NONE => {
                let expect_zero = self.unpack_uint()?;
                if expect_zero != 0 {
                    // TODO: handle as opaque
                    // Tenchically this is opaque and undefined but "recommended"
                    // that length is zero.
                    Err(INVALID_DATA.into())
                } else {
                    Ok(OpaqueAuth::None)
                }
            }
            AUTH_SYS => Ok(OpaqueAuth::Sys(self.unpack_auth_sys()?)),
            _ => Err(INVALID_DATA.into()),
        }
    }

    fn unpack_auth_sys(&mut self) -> Result<AuthSys> {
        let mut opaque: Bytes = self.unpack_opaque()?;
        Ok(AuthSys {
            stamp: opaque.unpack_uint()?,
            machine_name: opaque.unpack_opaque()?,
            uid: opaque.unpack_uint()?,
            gid: opaque.unpack_uint()?,
            gids: opaque.unpack_vec(|unpacker| unpacker.unpack_uint())?,
        })
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
    pub async fn run(&mut self) -> Result<()> {
        loop {
            let mut buf = BytesMut::new();
            read_packet(&mut self.connection, &mut buf, self.max_size).await?;
            let mut buf = buf.freeze();
            if buf.remaining() < 8 {
                println!("bad packet -- too short");
                continue;
            }

            let xid = buf.unpack_uint()?;
            let msg_type = buf.unpack_uint()?;
            match msg_type {
                CALL => println!("CB not implemented yet"),
                REPLY => {
                    let mut pending = self.pending.lock().unwrap();
                    let tx = pending.remove(&xid);
                    drop(pending);
                    match tx {
                        None => println!("unmatched xid {}", xid),
                        Some(tx) => {
                            tx.send(buf);
                        }
                    }
                }
                _ => println!("corrupt packet msg_type={}", msg_type),
            }
        }
    }
}

pub struct RpcClient {
    connection: tokio::sync::Mutex<WriteHalf<TcpStream>>,
    pending: Arc<Mutex<BTreeMap<u32, oneshot::Sender<Bytes>>>>,
}

impl RpcClient {
    pub fn new(connection: TcpStream) -> RpcClient {
        let (read, write) = tokio::io::split(connection);
        let pending = Arc::new(Mutex::new(BTreeMap::new()));

        let mut reader = RpcClientReceiver {
            connection: read,
            pending: pending.clone(),
            max_size: MAX_PACKET_SIZE,
        };

        tokio::spawn(async move {
            reader.run().await;
        });

        RpcClient {
            connection: tokio::sync::Mutex::new(write),
            pending,
        }
    }

    /// Returns a new xid (RPC transaction ID).
    ///
    /// According to the RFC:
    /// "The "xid" field is only used for clients matching reply
    /// messages with call messages or for servers detecting
    /// retransmissions; the service side cannot treat this id as any
    /// type of sequence number."
    pub fn next_xid() -> u32 {
        static XID : AtomicU32 = AtomicU32::new(0x58494430);
        XID.fetch_add(1, atomic::Ordering::Relaxed)
    }

    pub async fn call(&mut self, buf: impl Buf, xid: u32) -> io::Result<Bytes> {
        let (tx, rx) = oneshot::channel();
        let mut pending = self.pending.lock().unwrap();
        pending.insert(xid, tx);
        drop(pending);

        self.send(buf).await?;

        rx.await.map_err(|_| io::ErrorKind::Other.into())
    }

    async fn send(&mut self, mut buf: impl Buf) -> io::Result<()> {
        let mut connection = self.connection.lock().await;
        while buf.has_remaining() {
            connection.write_buf(&mut buf).await?;
        }

        Ok(())
    }

    pub fn check_header<B: Buf>(&mut self, buf: &mut B) -> Result<()> {
        let header = ReplyHeader::unpack_from(buf)?;
        match header {
            ReplyHeader::Accepted(AcceptedReply{stat, ..}) => {
                match stat {
                    AcceptedReplyStat::Success => Ok(()),
                    AcceptedReplyStat::ProgUnavail => Err(RPC_PROG_UNAVAIL.into()),
                    AcceptedReplyStat::ProgMismatch(_) => Err(RPC_PROG_MISMATCH.into()),
                    AcceptedReplyStat::ProcUnavail => Err(RPC_PROC_UNAVAIL.into()),
                    AcceptedReplyStat::GarbageArgs => Err(RPC_GARBAGE_ARGS.into()),
                    AcceptedReplyStat::SystemErr => Err(RPC_SYSTEM_ERR.into()),
                }
            },

            ReplyHeader::Denied(denied) => {
                match denied {
                    RejectedReply::RpcMismatch(_) => Err(RPC_REJECTED_MISMATCH.into()),
                    // TODO: unpack auth error
                    RejectedReply::AuthError(_) => Err(RPC_REJECTED_AUTH_ERROR.into())
                }
            }
        }
    }
}

