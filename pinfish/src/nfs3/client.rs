use crate::{
    mount,
    nfs3::{self, procs, Cookie3, DirOpArgs3, Filename3, NfsFh3, NfsPath3, Verifier3},
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

    pub async fn call_lookup(&self, dir: &NfsFh3, name: Filename3) -> Result<procs::LookupResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_LOOKUP);

        if let Some(rpc) = &self.nfs {
            let dir = dir.clone();
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

    pub async fn call_mkdir(&self, dir: &NfsFh3, name: Filename3) -> Result<procs::MkdirResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_MKDIR);

        if let Some(rpc) = &self.nfs {
            let attributes = nfs3::SetAttributes {
                mode: Some(0o755),
                ..Default::default()
            };
            let dir = dir.clone();
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

    pub async fn call_rename(
        &self,
        from_dir: &NfsFh3,
        from_name: Filename3,
        to_dir: &NfsFh3,
        to_name: Filename3,
    ) -> Result<procs::RenameResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_RENAME);

        if let Some(rpc) = &self.nfs {
            let from_dir = from_dir.clone();
            let to_dir = to_dir.clone();
            let rename = procs::Rename3Args {
                from: DirOpArgs3 {
                    dir: from_dir,
                    name: from_name,
                },
                to: DirOpArgs3 {
                    dir: to_dir,
                    name: to_name,
                },
            };
            rename.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::RenameResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_symlink(
        &self,
        dir: &NfsFh3,
        name: Filename3,
        data: NfsPath3,
    ) -> Result<procs::SymLinkResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_SYMLINK);

        if let Some(rpc) = &self.nfs {
            let attributes = nfs3::SetAttributes {
                mode: Some(0o755),
                ..Default::default()
            };
            let dir = dir.clone();
            let symlink = procs::SymLink3Args {
                symlink_where: DirOpArgs3 { dir, name },
                data: procs::SymLinkData3 { attributes, data },
            };
            symlink.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::SymLinkResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_getattr(&self, object: &NfsFh3) -> Result<procs::GetAttrResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_GETATTR);

        if let Some(rpc) = &self.nfs {
            let object = object.clone();
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

    pub async fn call_fsstat(&self, root: &NfsFh3) -> Result<procs::FsstatResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_FSSTAT);

        if let Some(rpc) = &self.nfs {
            let root = root.clone();
            let fsstat = procs::Fsstat3Args { root };
            fsstat.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::FsstatResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_fsinfo(&self, root: &NfsFh3) -> Result<procs::FsinfoResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_FSINFO);

        if let Some(rpc) = &self.nfs {
            let root = root.clone();
            let fsinfo = procs::Fsinfo3Args { root };
            fsinfo.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::FsinfoResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_pathconf(&self, root: &NfsFh3) -> Result<procs::PathconfResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_PATHCONF);

        if let Some(rpc) = &self.nfs {
            let root = root.clone();
            let pathconf = procs::Pathconf3Args { root };
            pathconf.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::PathconfResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_readlink(&self, symlink: &NfsFh3) -> Result<procs::ReadLinkResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_READLINK);

        if let Some(rpc) = &self.nfs {
            let symlink = symlink.clone();
            let readlink = procs::ReadLink3Args { symlink };
            readlink.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::ReadLinkResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_read(
        &self,
        file: &NfsFh3,
        offset: u64,
        count: u32,
    ) -> Result<procs::ReadResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_READ);

        if let Some(rpc) = &self.nfs {
            let file = file.clone();
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
        file: &NfsFh3,
        offset: u64,
        count: u32,
        data: Bytes,
    ) -> Result<procs::WriteResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_WRITE);

        if let Some(rpc) = &self.nfs {
            let stable = procs::StableHow::DataSync;
            let file = file.clone();
            let write = procs::Write3Args {
                file,
                offset,
                count,
                stable,
                data,
            };
            write.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::WriteResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_commit(
        &self,
        file: &NfsFh3,
        offset: u64,
        count: u32,
    ) -> Result<procs::CommitResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_COMMIT);

        if let Some(rpc) = &self.nfs {
            let file = file.clone();
            let commit = procs::Commit3Args {
                file,
                offset,
                count,
            };
            commit.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::CommitResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_remove(&self, dir: &NfsFh3, name: Filename3) -> Result<procs::RemoveResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_REMOVE);

        if let Some(rpc) = &self.nfs {
            let dir = dir.clone();
            let remove = procs::Remove3Args {
                object: DirOpArgs3 { dir, name },
            };
            remove.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::RemoveResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_link(
        &self,
        file: &NfsFh3,
        link_dir: &NfsFh3,
        link_name: Filename3,
    ) -> Result<procs::LinkResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_LINK);

        if let Some(rpc) = &self.nfs {
            let file = file.clone();
            let link_dir = link_dir.clone();
            let link = procs::Link3Args {
                file,
                link: DirOpArgs3 {
                    dir: link_dir,
                    name: link_name,
                },
            };
            link.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::LinkResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_readdir(
        &self,
        dir: &NfsFh3,
        cookie: Cookie3,
        verifier: Verifier3,
    ) -> Result<procs::ReaddirResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_READDIR);

        if let Some(rpc) = &self.nfs {
            let dir = dir.clone();
            let readdir = procs::Readdir3Args {
                dir,
                cookie,
                verifier,
                count: 65536,
            };

            readdir.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::ReaddirResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    pub async fn call_readdirplus(
        &self,
        dir: &NfsFh3,
        cookie: Cookie3,
        verifier: Verifier3,
    ) -> Result<procs::ReaddirPlusResult> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, Program::Nfs, nfs3::NFSPROC3_READDIRPLUS);

        if let Some(rpc) = &self.nfs {
            let dir = dir.clone();
            let readdirplus = procs::ReaddirPlus3Args {
                dir,
                cookie,
                verifier,
                dircount: 8192,
                maxcount: 65536,
            };

            readdirplus.pack_to(&mut buf);
            let buf = Self::finalize(buf);

            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            Ok(procs::ReaddirPlusResult::unpack_from(&mut response_buf)?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }
}
