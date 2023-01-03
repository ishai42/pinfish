use crate::{
    nfs4::{self, ops::ClientId4, ops::SequenceId4, sequence::ClientSequencer},
    result::{Result, INVALID_DATA, NOT_CONNECTED},
    rpc::{self, RpcClient},
    xdr::{PackTo, Packer, UnpackFrom},
};
use bytes::{Buf, Bytes, BytesMut};
use std::borrow::BorrowMut;
use tokio::net::TcpStream;
use core::cell::Cell;

pub struct NfsClient {
    /// Server address in host:port format
    server: String,

    /// Active connection
    rpc: Option<RpcClient>,

    /// Client ID returned from EXCHANGE_ID
    pub client_id: Cell<ClientId4>,

    /// Sequence ID returned from EXCHANGE_ID
    pub sequence_id: Cell<SequenceId4>,

    seq: ClientSequencer,
}

impl NfsClient {
    /// Consructs a new `NfsClient`
    pub fn new(server: &str) -> NfsClient {
        NfsClient {
            server: server.into(),
            rpc: None,
            client_id: Cell::new(0),
            sequence_id: Cell::new(0),
            seq: ClientSequencer::new(64)
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
                        owner_id: Vec::from(*b"owner/id/string"),
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
            compound
                .arg_array
                .push(nfs4::ops::ArgOp4::CreateSession(nfs4::ops::CreateSession4Args {
                    client_id: self.client_id.get(),
                    sequence: self.sequence_id.get(),
                    flags: nfs4::ops::CREATE_SESSION4_FLAG_PERSIST,
                    fore_chan_attrs: nfs4::ops::ChannelAttrs4{
                        header_pad_size: 0,
                        max_request_size: 0x100800,
                        max_response_size:  0x100800,
                        max_response_size_cached: 0x1800,
                        max_operation: 8,
                        max_requests: 64,
                        rdma_ird: None,
                    },
                    back_chan_attrs: nfs4::ops::ChannelAttrs4{
                        header_pad_size: 0,
                        max_request_size: 0x1000,
                        max_response_size:  0x1000,
                        max_response_size_cached: 0,
                        max_operation: 2,
                        max_requests: 16,
                        rdma_ird: None,
                    },
                    cb_program: 0x40000000,
                    sec_params: vec![nfs4::ops::CallbackSecParams4::AuthNone],
                }));

            compound.pack_to(&mut buf);

            let buf = Self::finalize(buf);
            let mut response_buf = rpc.call(buf, xid).await?;
            rpc.check_header(&mut response_buf)?;
            let resp = nfs4::ops::CompoundResult::unpack_from(&mut response_buf)?;
            if resp.status != nfs4::NFS4_OK {
                return Err(resp.status.into());
            }
            if let Some(nfs4::ops::ResultOp4::CreateSession(reply)) = resp.result_array.first() {
                let _reply = reply.as_ref()?;

                // TODO: either return the OK reply or record result for the session.

                Ok(())
            } else {
                Err(INVALID_DATA.into())
            }
        } else {
            Err(NOT_CONNECTED.into())
        }
    }


}
