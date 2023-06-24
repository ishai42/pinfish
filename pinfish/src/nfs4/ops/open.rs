use super::{Verifier4, Bitmap4, FileAttributes, SequenceId4, OpenOwner4, StateId4, ChangeInfo4, OpenDelegation4};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;


#[derive(PackTo, UnpackFrom, Debug)]
pub enum OpenFlag4 {
    NoCreate,
    Create(CreateHow4),
}

#[derive(PackTo, UnpackFrom, Debug)]
pub enum CreateHow4 {
    Unchecked(FileAttributes),
    Guarded(FileAttributes),
    Exclusive(Verifier4),
}

#[derive(PackTo, UnpackFrom, Debug)]
pub enum OpenClaim4 {
    #[xdr(0)]
    Null(String),
    //    Previous(OpenDelegationType4),
    //    DelegateCur(OpenClaimDelegateCur4),
    //    DelegatePrev(String),
    #[xdr(4)]
    FileHandle,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Open4Args {
    /// The "seqid" field of the request is not used in NFSv4.1
    pub seqid: SequenceId4,
    pub share_access: u32,
    pub share_deny: u32,
    pub owner: OpenOwner4,
    pub how: OpenFlag4,
    pub claim: OpenClaim4,
}

#[derive(PackTo, UnpackFrom, Debug, Clone)]
pub struct Open4ResOk {
    pub state_id: StateId4,
    pub change_info: ChangeInfo4,
    pub result_flags: u32,
    pub attr_set: Bitmap4,
    pub delegation: OpenDelegation4,
}
