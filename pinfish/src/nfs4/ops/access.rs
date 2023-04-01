use crate::xdr;
use pinfish_macros::{PackTo, UnpackFrom};

/// Read data from file or read directory
pub const ACCESS4_READ: u32 = 0x00000001;
/// Lookup a name in a directory
pub const ACCESS4_LOOKUP: u32 = 0x00000002;
/// Rewrite existing file data or modify existing directory entries
pub const ACCESS4_MODIFY: u32 = 0x00000004;
/// Write new data or add directory entries
pub const ACCESS4_EXTEND: u32 = 0x00000008;
/// Delete an existing directory entry
pub const ACCESS4_DELETE: u32 = 0x00000010;
/// Execute a regular file
pub const ACCESS4_EXECUTE: u32 = 0x00000020;

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Access4Args {
    /// Access rights to be checked.  This is a bitwise combination of
    /// bits from `ACCESS4_READ`, `ACCESS4_LOOKUP`, `ACCESS4_MODIFY`,
    /// `ACCESS4_EXTEND`, `ACCESS4_DELETE`, and `ACCESS4_EXECUTE`
    pub access: u32,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Access4ResOk {
    /// Access rights the server can verify
    pub supported: u32,
    /// Access rights available to the user for the filehandle
    pub access: u32,
}
