use crate::{
    nfs3::{DirOpArgs3, PostOpAttributes, PostOpFh3, SetAttributes, WccData, SpecData3},
    xdr,
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct DeviceData3 {
    pub attributes: SetAttributes,
    pub spec: SpecData3,
}

#[derive(PackTo, Debug)]
pub enum MknodData3 {
    #[xdr(4)]
    Chr(DeviceData3),
    #[xdr(3)]
    Blk(DeviceData3),
    #[xdr(6)]
    Sock(SetAttributes),
    #[xdr(7)]
    Fifo(SetAttributes),
}

#[derive(PackTo, Debug)]
pub struct Mknod3Args {
    pub mknod_where: DirOpArgs3,
    pub what: MknodData3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Mknod3ResOk {
    pub obj: PostOpFh3,
    pub attributes: PostOpAttributes,
    pub wcc_data: WccData,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Mknod3ResFail {
    pub dir_wcc: WccData,
}

pub type MknodResult = Result<Mknod3ResOk, (u32, Mknod3ResFail)>;
