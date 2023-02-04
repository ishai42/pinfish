use crate::{
    nfs4::{self,
           ops::{ClientId4, SequenceId4,SessionId4, NfsFh4 },
           sequence::{ClientSequence, ClientSequencer},
    },
    result::{Result, INVALID_DATA, NOT_CONNECTED},
    rpc::{self, RpcClient},
    xdr::{PackTo, Packer, UnpackFrom},
};
use bytes::{Buf, Bytes, BytesMut};
use core::cell::Cell;
use std::borrow::BorrowMut;
use tokio::net::TcpStream;
use std::collections::btree_map::BTreeMap;

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
            root_node: std::sync::Mutex::new(Default::default())
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
                    flags: nfs4::ops::EXCHGID4_FLAG_USE_PNFS_MDS
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
        nfs4::ops::ArgOp4::Sequence(nfs4::ops::Sequence4Args{
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
            compound.arg_array.push(self.new_sequence_op(&sequence, false));
            compound.arg_array.push(nfs4::ops::ArgOp4::ReclaimComplete(nfs4::ops::ReclaimComplete4Args{ one_fs: false }));

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
            compound.arg_array.push(self.new_sequence_op(&sequence, false));
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
            dbg!(&resp);
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
            compound.arg_array.push(self.new_sequence_op(&sequence, false));
            compound.arg_array.push(nfs4::ops::ArgOp4::PutFh(nfs4::ops::PutFh4Args{
                object: parent.clone(),
            }));
            compound.arg_array.push(nfs4::ops::ArgOp4::Lookup(nfs4::ops::Lookup4Args{
                objname: name.into()
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

}
