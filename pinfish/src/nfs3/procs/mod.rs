use crate::{
    result::{ErrorCode, Result},
    xdr::{UnpackFrom, Unpacker},
};

macro_rules! pub_use{
    ($($name:ident),+) => { $(mod $name; pub use $name::*;)+ }
}

pub_use!(lookup, create, getattr, setattr, access, readlink, read, write);
pub_use!(mkdir, symlink, mknod, remove, rmdir, rename, link, readdir);
pub_use!(readdirplus, fsstat, fsinfo, pathconf, commit);

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
