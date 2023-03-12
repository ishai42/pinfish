use crate::{
    nfs4::{
        self,
        attr::{self, Bitmap4},
        ops::{
            ClientId4, Cookie4, NfsFh4, Open4ResOk, Read4ResOk, ReadDir4ResOk, SequenceId4,
            SessionId4, StateId4, Verifier4,
        },
        sequence::{ClientSequence, ClientSequencer},
    },
    result::{Result, INVALID_DATA, NOT_CONNECTED},
    rpc::{self, RpcClient},
    xdr::{PackTo, Packer, UnpackFrom},
};
use bytes::{Buf, Bytes, BytesMut};
use core::cell::Cell;
use std::borrow::BorrowMut;
use std::collections::btree_map::BTreeMap;
use tokio::net::TcpStream;

#[derive(Default)]
pub struct ClientFsDirNode {
    pub fh: NfsFh4,
    pub children: BTreeMap<String, ClientFsNode>,
}

pub struct ClientFsDirFileNode {
    pub fh: NfsFh4,
}

pub enum ClientFsNode {
    File(ClientFsDirFileNode),
    Dir(ClientFsDirNode),
}

pub struct NfsClient {
    /// Server address in host:port format
    server: String,

    /// Active connection
    rpc: Option<RpcClient>,

    /// Client ID returned from EXCHANGE_ID
    pub client_id: Cell<ClientId4>,

    /// Sequence ID returned from EXCHANGE_ID
    pub sequence_id: Cell<SequenceId4>,

    /// Session ID retruend from CREATE_SESSION
    pub session_id: Cell<SessionId4>,

    /// cached remote file and directory attributes
    pub root_node: std::sync::Mutex<ClientFsDirNode>,

    /// Generator for slot & sequence pairs.
    pub seq: ClientSequencer,
}

impl NfsClient {
    /// Consructs a new `NfsClient`
    pub fn new(server: &str) -> NfsClient {
        NfsClient {
            server: server.into(),
            rpc: None,
            client_id: Cell::new(0),
            sequence_id: Cell::new(0),
            session_id: Cell::new(Default::default()),
            seq: ClientSequencer::new(64),
            root_node: std::sync::Mutex::new(Default::default()),
        }
    }

    /// Connects the client
    pub async fn connect(&mut self) -> Result<()> {
        let connection = TcpStream::connect(&self.server).await?;
        self.rpc = Some(RpcClient::new(connection));

        Ok(())
    }

    fn new_rpc_header(&self, proc: u32) -> rpc::CallHeader {
        rpc::CallHeader {
            prog: nfs4::PROG_NFS,
            vers: 4,
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

    fn new_buf_with_call_header(&self, xid: u32, proc: u32) -> BytesMut {
        let mut buf = self.new_buf();
        buf.pack_uint(xid);
        self.new_rpc_header(proc).pack_to(&mut buf);

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

    /// Make a NULL RPC call
    pub async fn null_call(&self) -> Result<Bytes> {
        let xid = RpcClient::next_xid();
        let buf = self.new_buf_with_call_header(xid, nfs4::PROC_NULL);
        if let Some(rpc) = &self.rpc {
            let buf = Self::finalize(buf);
            Ok(rpc.call(buf, xid).await?)
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    /// Make an EXCHANGE_ID call and process the result
    pub async fn exchange_id_call(&self) -> Result<()> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, nfs4::PROC_COMPOUND);
        if let Some(rpc) = &self.rpc {
            let mut compound = nfs4::ops::Compound::new();
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::ExchangeId(nfs4::ops::ExchangeId4Args {
                    client_owner: nfs4::ops::ClientOwner4 {
                        verifier: 0,
                        owner_id: Vec::from(*b"owner/id/string/2"),
                    },
                    flags: nfs4::ops::EXCHGID4_FLAG_BIND_PRINC_STATEID
                        | nfs4::ops::EXCHGID4_FLAG_SUPP_MOVED_MIGR
                        | nfs4::ops::EXCHGID4_FLAG_SUPP_MOVED_REFER,
                    state_protect: nfs4::ops::StateProtect4A::None,
                    client_impl_id: None,
                }));

            compound.pack_to(&mut buf);

            let buf = Self::finalize(buf);
            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf)?;
            if resp.status != nfs4::NFS4_OK {
                return Err(resp.status.into());
            }
            if let Some(nfs4::ops::ResultOp4::ExchangeId(reply)) = resp.result_array.first() {
                let reply = reply.as_ref()?;
                self.client_id.set(reply.client_id);
                self.sequence_id.set(reply.sequence_id);

                Ok(())
            } else {
                Err(INVALID_DATA.into())
            }
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    /// Make a CREATE_SESSION call and process the result
    pub async fn create_session_call(&self) -> Result<()> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, nfs4::PROC_COMPOUND);
        if let Some(rpc) = &self.rpc {
            let mut compound = nfs4::ops::Compound::new();
            compound.arg_array.push(nfs4::ops::ArgOp4::CreateSession(
                nfs4::ops::CreateSession4Args {
                    client_id: self.client_id.get(),
                    sequence: self.sequence_id.get(),
                    flags: nfs4::ops::CREATE_SESSION4_FLAG_PERSIST,
                    fore_chan_attrs: nfs4::ops::ChannelAttrs4 {
                        header_pad_size: 0,
                        max_request_size: 0x100800,
                        max_response_size: 0x100800,
                        max_response_size_cached: 0x1800,
                        max_operation: 8,
                        max_requests: 64,
                        rdma_ird: None,
                    },
                    back_chan_attrs: nfs4::ops::ChannelAttrs4 {
                        header_pad_size: 0,
                        max_request_size: 0x1000,
                        max_response_size: 0x1000,
                        max_response_size_cached: 0,
                        max_operation: 2,
                        max_requests: 16,
                        rdma_ird: None,
                    },
                    cb_program: 0x40000000,
                    sec_params: vec![nfs4::ops::CallbackSecParams4::AuthNone],
                },
            ));

            compound.pack_to(&mut buf);

            let buf = Self::finalize(buf);
            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf)?;
            if resp.status != nfs4::NFS4_OK {
                return Err(resp.status.into());
            }
            if let Some(nfs4::ops::ResultOp4::CreateSession(reply)) = resp.result_array.first() {
                let reply = reply.as_ref()?;

                self.session_id.set(reply.session_id);

                Ok(())
            } else {
                Err(INVALID_DATA.into())
            }
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    fn new_sequence_op(&self, sequence: &ClientSequence, cache_this: bool) -> nfs4::ops::ArgOp4 {
        nfs4::ops::ArgOp4::Sequence(nfs4::ops::Sequence4Args {
            session_id: self.session_id.get(),
            sequence_id: sequence.info.sequence,
            slot_id: sequence.info.slot,
            highest_slot_id: self.seq.get_max(),
            cache_this,
        })
    }

    /// Make a RECLAIM_COMPLETE call and process the result
    pub async fn send_reclaim_complete(&self) -> Result<()> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, nfs4::PROC_COMPOUND);

        if let Some(rpc) = &self.rpc {
            let mut compound = nfs4::ops::Compound::new();
            let sequence = self.seq.get_seq().await;
            compound
                .arg_array
                .push(self.new_sequence_op(&sequence, false));
            compound.arg_array.push(nfs4::ops::ArgOp4::ReclaimComplete(
                nfs4::ops::ReclaimComplete4Args { one_fs: false },
            ));

            compound.pack_to(&mut buf);

            let buf = Self::finalize(buf);
            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf)?;
            if resp.status != nfs4::NFS4_OK {
                return Err(resp.status.into());
            }

            if let nfs4::ops::ResultOp4::ReclaimComplete(reply) = &resp.result_array[1] {
                Ok((*reply)?)
            } else {
                Err(INVALID_DATA.into())
            }
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    fn get_root_fh(&self) -> NfsFh4 {
        self.root_node.lock().unwrap().fh.clone()
    }

    /// Make a PUTROOTFH | GETFH call and process the result
    pub async fn send_putrootfh(&self) -> Result<NfsFh4> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, nfs4::PROC_COMPOUND);

        if let Some(rpc) = &self.rpc {
            let mut compound = nfs4::ops::Compound::new();
            let sequence = self.seq.get_seq().await;
            compound
                .arg_array
                .push(self.new_sequence_op(&sequence, false));
            compound.arg_array.push(nfs4::ops::ArgOp4::PutRootFh);
            compound.arg_array.push(nfs4::ops::ArgOp4::GetFh);

            compound.pack_to(&mut buf);

            let buf = Self::finalize(buf);
            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf)?;
            if resp.status != nfs4::NFS4_OK {
                return Err(resp.status.into());
            }

            if let nfs4::ops::ResultOp4::GetFh(reply) = &resp.result_array[2] {
                let mut root_node = self.root_node.lock().unwrap();
                root_node.fh = reply.as_ref()?.object.clone();
                Ok(root_node.fh.clone())
            } else {
                Err(INVALID_DATA.into())
            }
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    /// Make a PUTFH | LOOKUP | GETFH call and process the result
    pub async fn send_lookup(&self, parent: &NfsFh4, name: &str) -> Result<NfsFh4> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, nfs4::PROC_COMPOUND);

        if let Some(rpc) = &self.rpc {
            let mut compound = nfs4::ops::Compound::new();
            let sequence = self.seq.get_seq().await;
            compound
                .arg_array
                .push(self.new_sequence_op(&sequence, false));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::PutFh(nfs4::ops::PutFh4Args {
                    object: parent.clone(),
                }));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::Lookup(nfs4::ops::Lookup4Args {
                    objname: name.into(),
                }));
            compound.arg_array.push(nfs4::ops::ArgOp4::GetFh);

            compound.pack_to(&mut buf);

            let buf = Self::finalize(buf);
            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf)?;
            if resp.status != nfs4::NFS4_OK {
                return Err(resp.status.into());
            }

            if let nfs4::ops::ResultOp4::GetFh(reply) = &resp.result_array[3] {
                let mut root_node = self.root_node.lock().unwrap();
                root_node.fh = reply.as_ref()?.object.clone();
                Ok(root_node.fh.clone())
            } else {
                Err(INVALID_DATA.into())
            }
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    /// Make a PUTFH | CREATE | GETFH call
    pub async fn mkdir(&self, parent: &NfsFh4, name: &str) -> Result<NfsFh4> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, nfs4::PROC_COMPOUND);

        if let Some(rpc) = &self.rpc {
            let mut compound = nfs4::ops::Compound::new();
            let mut attributes = nfs4::attr::FileAttributes::new();
            attributes.mode = Some(0o775);
            let sequence = self.seq.get_seq().await;
            compound
                .arg_array
                .push(self.new_sequence_op(&sequence, false));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::PutFh(nfs4::ops::PutFh4Args {
                    object: parent.clone(),
                }));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::Create(nfs4::ops::Create4Args {
                    objtype: nfs4::ops::CreateType4::Directory,
                    component: name.into(),
                    attributes,
                }));
            compound.arg_array.push(nfs4::ops::ArgOp4::GetFh);

            compound.pack_to(&mut buf);

            let buf = Self::finalize(buf);
            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf)?;
            if resp.status != nfs4::NFS4_OK {
                return Err(resp.status.into());
            }

            if let nfs4::ops::ResultOp4::GetFh(reply) = &resp.result_array[3] {
                let mut root_node = self.root_node.lock().unwrap();
                root_node.fh = reply.as_ref()?.object.clone();
                Ok(root_node.fh.clone())
            } else {
                Err(INVALID_DATA.into())
            }
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    /// Make a PUTFH | REMOVE call and process the result
    pub async fn remove(&self, parent: &NfsFh4, name: &str) -> Result<()> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, nfs4::PROC_COMPOUND);

        if let Some(rpc) = &self.rpc {
            let mut compound = nfs4::ops::Compound::new();
            let sequence = self.seq.get_seq().await;
            compound
                .arg_array
                .push(self.new_sequence_op(&sequence, false));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::PutFh(nfs4::ops::PutFh4Args {
                    object: parent.clone(),
                }));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::Remove(nfs4::ops::Remove4Args {
                    target: name.into(),
                }));

            compound.pack_to(&mut buf);

            let buf = Self::finalize(buf);
            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf)?;
            if resp.status != nfs4::NFS4_OK {
                return Err(resp.status.into());
            }

            if let nfs4::ops::ResultOp4::Remove(_) = &resp.result_array[2] {
                Ok(())
            } else {
                Err(INVALID_DATA.into())
            }
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    /// Make a PUTFH | READDIR call and return the result
    pub async fn readdir(
        &self,
        dir: &NfsFh4,
        cookie: Cookie4,
        verifier: Verifier4,
    ) -> Result<ReadDir4ResOk> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, nfs4::PROC_COMPOUND);

        let mut attr_request = Bitmap4::new();
        attr_request.set(attr::TYPE);
        attr_request.set(attr::SIZE);
        attr_request.set(attr::MODE);
        attr_request.set(attr::OWNER);

        if let Some(rpc) = &self.rpc {
            let mut compound = nfs4::ops::Compound::new();
            let sequence = self.seq.get_seq().await;
            compound
                .arg_array
                .push(self.new_sequence_op(&sequence, false));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::PutFh(nfs4::ops::PutFh4Args {
                    object: dir.clone(),
                }));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::ReadDir(nfs4::ops::ReadDir4Args {
                    cookie,
                    verifier,
                    dir_count: 8170,
                    max_count: 32680,
                    attr_request,
                }));

            compound.pack_to(&mut buf);

            let buf = Self::finalize(buf);
            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let mut resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf)?;
            if resp.status != nfs4::NFS4_OK {
                return Err(resp.status.into());
            }

            if let nfs4::ops::ResultOp4::ReadDir(reply) = &mut resp.result_array[2] {
                let mut readdir = Err(crate::result::INTERNAL_ERROR);
                std::mem::swap(&mut *reply, &mut readdir);
                Ok(readdir?)
            } else {
                Err(INVALID_DATA.into())
            }
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    /// Make a PUTFH | OPEN call and return the result
    pub async fn open_by_id(
        &self,
        file: &NfsFh4,
        share_access: u32,
        share_deny: u32,
    ) -> Result<Open4ResOk> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, nfs4::PROC_COMPOUND);

        if let Some(rpc) = &self.rpc {
            let mut compound = nfs4::ops::Compound::new();
            let sequence = self.seq.get_seq().await;
            compound
                .arg_array
                .push(self.new_sequence_op(&sequence, false));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::PutFh(nfs4::ops::PutFh4Args {
                    object: file.clone(),
                }));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::Open(nfs4::ops::Open4Args {
                    seqid: 0,
                    share_access,
                    share_deny,
                    owner: nfs4::ops::OpenOwner4 {
                        client_id: self.client_id.get(),
                        owner: "foo".into(),
                    },
                    how: nfs4::ops::OpenFlag4::NoCreate,
                    claim: nfs4::ops::OpenClaim4::FileHandle,
                }));

            compound.pack_to(&mut buf);

            let buf = Self::finalize(buf);
            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let mut resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf)?;
            if resp.status != nfs4::NFS4_OK {
                return Err(resp.status.into());
            }

            if let nfs4::ops::ResultOp4::Open(reply) = &mut resp.result_array[2] {
                Ok(reply.clone()?)
            } else {
                Err(INVALID_DATA.into())
            }
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    /// Make a PUTFH | READ call and return the result
    pub async fn read(
        &self,
        fh: &NfsFh4,
        state_id: &StateId4,
        offset: u64,
        count: u32,
    ) -> Result<Read4ResOk> {
        let xid = RpcClient::next_xid();
        let mut buf = self.new_buf_with_call_header(xid, nfs4::PROC_COMPOUND);

        if let Some(rpc) = &self.rpc {
            let mut compound = nfs4::ops::Compound::new();
            let sequence = self.seq.get_seq().await;
            compound
                .arg_array
                .push(self.new_sequence_op(&sequence, false));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::PutFh(nfs4::ops::PutFh4Args {
                    object: fh.clone(),
                }));
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::Read(nfs4::ops::Read4Args {
                    state_id: state_id.clone(),
                    offset,
                    count,
                }));

            compound.pack_to(&mut buf);

            let buf = Self::finalize(buf);
            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let mut resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf)?;
            if resp.status != nfs4::NFS4_OK {
                return Err(resp.status.into());
            }

            if let nfs4::ops::ResultOp4::Read(reply) = &mut resp.result_array[2] {
                Ok(reply.clone()?)
            } else {
                Err(INVALID_DATA.into())
            }
        } else {
            Err(NOT_CONNECTED.into())
        }
    }

    /// Returns the root FH memory or from server.
    pub async fn get_root(&self) -> Result<NfsFh4> {
        let root = self.get_root_fh();
        if root.len() > 0 {
            Ok(root)
        } else {
            Ok(self.send_putrootfh().await?)
        }
    }

    /// Resolves a path into FH, possibly using cached resolution
    pub async fn resolve_path(&self, path: &str) -> Result<NfsFh4> {
        let mut fh = self.get_root().await?;
        if path.is_empty() {
            return Ok(fh);
        }

        for component in path.split('/') {
            fh = self.send_lookup(&fh, component).await?;
        }

        Ok(fh)
    }
}
