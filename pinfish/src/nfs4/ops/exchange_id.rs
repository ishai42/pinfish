use super::{Verifier4, NfsTime4, ClientId4, SequenceId4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;

#[derive(PackTo, UnpackFrom, Debug)]
pub enum StateProtect4R {
    None,
    // TODO: MachCred
    // TODO: Ssv
}

#[derive(PackTo, UnpackFrom, Debug)]
pub enum StateProtect4A {
    None,
    // TODO: MachCred
    // TODO: Ssv
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct ClientOwner4 {
    pub verifier: Verifier4,
    pub owner_id: Vec<u8>,
}


#[derive(PackTo, UnpackFrom, Debug)]
pub struct NfsImplId4 {
    pub domain: String,
    pub name: String,
    pub date: NfsTime4,
}


/// 18.35 EXCHANGE_ID4args
#[derive(PackTo, UnpackFrom, Debug)]
pub struct ExchangeId4Args {
    pub client_owner: ClientOwner4,
    pub flags: u32,
    pub state_protect: StateProtect4A,
    pub client_impl_id: Option<NfsImplId4>,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct ServerOwner4 {
    pub minor_id: u64,
    pub major_id: Vec<u8>,
}


/// 18.35.2 EXCHANGE_ID4resok
#[derive(PackTo, UnpackFrom, Debug)]
pub struct ExchangeId4ResOk {
    pub client_id: ClientId4,
    pub sequence_id: SequenceId4,
    pub flags: u32,
    pub state_protect: StateProtect4R,
    pub server_owner: ServerOwner4,
    pub server_scope: Vec<u8>,
    pub server_impl_id: Option<NfsImplId4>,
}
