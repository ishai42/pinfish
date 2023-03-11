use crate::{
    mount,
    nfs3::{self, procs, DirOpArgs3, Filename3, NfsFh3},
    portmap,
    result::{Result, NOT_CONNECTED},
    rpc::{self, RpcClient},
    xdr::{PackTo, Packer, UnpackFrom},
};
//use core::cell::Cell;
use bytes::{Buf, Bytes, BytesMut};
use std::borrow::BorrowMut;
use tokio::net::TcpStream;

pub struct NfsClient {
    /// Server address
    server: String,

    /// port mapper RPC client
    portmap: Option<RpcClient>,

    /// Mount RPC client
    mount: Option<RpcClient>,

    /// NFSv3 RPC client
    nfs: Option<RpcClient>,

    mount_port: u16,
    nfs_port: u16,

    root_fh: std::sync::Mutex<NfsFh3>,
}

#[derive(Clone, Copy)]
pub enum Program {
    Portmap,
    // Bind,
    Mount,
    Nfs,
}

impl Program {
    pub const fn prog(&self) -> u32 {
        match self {
            Program::Portmap => portmap::PMAP_PROG,
            Program::Mount => mount::PROGRAM,
            Program::Nfs => nfs3::PROG_NFS,
        }
    }

    pub const fn vers(&self) -> u32 {
        match self {
            Program::Portmap => 2, // RFC 1833
            Program::Mount => 3,
            Program::Nfs => 3,
        }
    }
}

impl NfsClient {
    /// Consructs a new `NfsClient`
    pub fn new(server: &str) -> NfsClient {
        NfsClient {
            server: server.into(),
            portmap: None,
            mount: None,
            nfs: None,
            mount_port: 0,
            nfs_port: 0,
            root_fh: std::sync::Mutex::new(Default::default()),
        }
    }

    /// Connects the portmap client
    async fn connect_portmap(&mut self) -> Result<()> {
        let host = std::format!("{}:{}", &self.server, portmap::PORT);
        let connection = TcpStream::connect(host).await?;
        self.portmap = Some(RpcClient::new(connection));

        Ok(())
    }

    /// Connects the portmap client if not yet connected
    async fn connect_portmap_if_needed(&mut self) -> Result<()> {
        if let None = &self.portmap {
            Ok(self.connect_portmap().await?)
        } else {
            Ok(())
        }
    }

    fn new_rpc_header(&self, prog: Program, proc: u32) -> rpc::CallHeader {
        rpc::CallHeader {
            prog: prog.prog(),
            vers: prog.vers(),
            proc,
            cred: rpc::OpaqueAuth::new_sys(1, Bytes::from_static(b"blah"), 0, 0, Vec::new()),
            verf: rpc::OpaqueAuth::new_none(),
        }
    }

    /// Constructs a new buffer with placeholder for the RPC frag marker
    fn new_buf(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        buf.pack_uint(0); // placeholder for frag

        buf
    }

    fn new_buf_with_call_header(&self, xid: u32, prog: Program, proc: u32) -> BytesMut {
        let mut buf = self.new_buf();
        buf.pack_uint(xid);
        self.new_rpc_header(prog, proc).pack_to(&mut buf);

        buf
    }

    /// Cosumes `buf` and updates frag size, returns frozen buffer
    fn finalize(mut buf: BytesMut) -> Bytes {
        let frag_size = (buf.remaining() - 4) as u32;
        let frag_size = frag_size | 0x80000000;
        {
            let borrow: &mut [u8] = buf.borrow_mut();
            (&mut borrow[0..4]).pack_uint(frag_size);
        }

        buf.freeze()
    }

    async fn call_portmap_get_port(&self, program: Program) -> Result<u32> {
        let xid = RpcClient::next_xid();
        let mut buf =
            self.new_buf_with_call_header(xid, Program::Portmap, portmap::PMAPPROC_GETPORT);

        if let Some(rpc) = &self.portmap {
            let mapping = portmap::Mapping {
                prog: program.prog(),
                vers: program.vers(),
                prot: portmap::IPPROTO_TCP,
                port: 0,
            };

            mapping.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(u32::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    async fn portmap_get_port(&mut self, program: Program) -> Result<u32> {
        self.connect_portmap_if_needed().await?;
        Ok(self.call_portmap_get_port(program).await?)
    }

    pub async fn connect_mount(&mut self) -> Result<()> {
        if self.mount_port == 0 {
            let port = self.portmap_get_port(Program::Mount).await?;
            self.mount_port = port as u16;
        }

        let host = std::format!("{}:{}", &self.server, self.mount_port);
        let connection = TcpStream::connect(host).await?;
        self.mount = Some(RpcClient::new(connection));

        Ok(())
    }

    pub async fn connect_nfs(&mut self) -> Result<()> {
        if self.nfs_port == 0 {
            let port = self.portmap_get_port(Program::Nfs).await?;
            self.nfs_port = port as u16;
        }

        let host = std::format!("{}:{}", &self.server, self.nfs_port);
        let connection = TcpStream::connect(host).await?;
        self.nfs = Some(RpcClient::new(connection));

        Ok(())
    }

    pub async fn call_mount(&self, path: &str) -> Result<mount::MountResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Mount, mount::MOUNTPROC3_MNT);

        if let Some(rpc) = &self.mount {
            buf.pack_string(path);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let res_ok = mount::MountResult::unpack_from(&mut response_buf)??;

            self.root_fh.lock().unwrap().data = res_ok.handle.data.clone();

            Ok(Ok(res_ok))
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_lookup(&self, dir: NfsFh3, name: Filename3) -> Result<procs::LookupResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_LOOKUP);

        if let Some(rpc) = &self.nfs {
            let lookup = procs::Lookup3Args {
                what: DirOpArgs3 { dir, name },
            };
            lookup.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::LookupResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_mkdir(&self, dir: NfsFh3, name: Filename3) -> Result<procs::MkdirResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_MKDIR);

        if let Some(rpc) = &self.nfs {
            let attributes = nfs3::SetAttributes {
                mode: Some(0o755),
                ..Default::default()
            };
            let mkdir = procs::Mkdir3Args {
                mkdir_where: DirOpArgs3 { dir, name },
                attributes,
            };
            mkdir.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::MkdirResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_getattr(&self, object: NfsFh3) -> Result<procs::GetAttrResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_GETATTR);

        if let Some(rpc) = &self.nfs {
            let getattr = procs::GetAttr3Args { object };
            getattr.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::GetAttrResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_read(
        &self,
        file: NfsFh3,
        offset: u64,
        count: u32,
    ) -> Result<procs::ReadResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_READ);

        if let Some(rpc) = &self.nfs {
            let getattr = procs::Read3Args {
                file,
                offset,
                count,
            };
            getattr.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::ReadResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_write(
        &self,
        file: NfsFh3,
        offset: u64,
        count: u32,
        data: Bytes,
    ) -> Result<procs::WriteResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_WRITE);

        if let Some(rpc) = &self.nfs {
            let stable = procs::StableHow::DataSync;
            let getattr = procs::Write3Args {
                file,
                offset,
                count,
                stable,
                data,
            };
            getattr.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::WriteResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }
}
