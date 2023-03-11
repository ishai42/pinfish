use crate::{
    result::{ErrorCode, Result},
    xdr::{UnpackFrom, Unpacker},
};

mod lookup;
pub use lookup::*;

mod create;
pub use create::*;

mod getattr;
pub use getattr::*;

mod setattr;
pub use setattr::*;

mod access;
pub use access::*;

mod readlink;
pub use readlink::*;

mod read;
pub use read::*;

mod write;
pub use write::*;

mod mkdir;
pub use mkdir::*;

impl<T, E, B> UnpackFrom<B> for core::result::Result<T, (u32, E)>
where
    T: core::fmt::Debug + UnpackFrom<B>,
    E: core::fmt::Debug + UnpackFrom<B>,
    B: Unpacker,
{
    fn unpack_from(buf: &mut B) -> Result<Self> {
        let n = u32::unpack_from(buf)?;
        match n {
            0 => Ok(Ok(T::unpack_from(buf)?)),
            _ => Ok(Err((n, E::unpack_from(buf)?))),
        }
    }
}

impl<T: core::fmt::Debug + UnpackFrom<B>, B: Unpacker> UnpackFrom<B>
    for core::result::Result<T, ErrorCode>
{
    fn unpack_from(buf: &mut B) -> Result<Self> {
        let n = u32::unpack_from(buf)?;
        match n {
            0 => Ok(Ok(T::unpack_from(buf)?)),
            _ => Ok(Err(ErrorCode::from(n))),
        }
    }
}
