use std::num::NonZeroU32;

/// 32-bit error code, RFC5661 error codes kept at same values,
/// anything else is translated to constants defined in this module
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ErrorCode(NonZeroU32);

impl ErrorCode {
    /// Create a new error code.  Panics if code is zero
    pub(crate) const fn new(code: u32) -> Self {
        if let Some(code) = NonZeroU32::new(code) {
            ErrorCode(code)
        } else {
            panic!("zero error code");
        }
    }

    /// Returns the 32-bit code as a u32
    pub const fn get(&self) -> u32 {
        self.0.get()
    }
}

impl From<u32> for ErrorCode {
    fn from(n: u32) -> ErrorCode {
        ErrorCode::new(if n == 0 { INTERNAL_ERROR } else { n })
    }
}

impl From<std::io::Error> for ErrorCode {
    fn from(err: std::io::Error) -> ErrorCode {
        match err.kind() {
            std::io::ErrorKind::ConnectionRefused => CONNECTION_REFUSED,
            std::io::ErrorKind::ConnectionReset => CONNECTION_RESET,
            //std::io::ErrorKind::HostUnreachable => HOST_UNREACHABLE,
            //std::io::ErrorKind::NetworkUnreachable => NETWORK_UNREACHABLE,
            std::io::ErrorKind::ConnectionAborted => CONNECTION_ABORTED,
            std::io::ErrorKind::NotConnected => NOT_CONNECTED,
            std::io::ErrorKind::InvalidData => INVALID_DATA,
            _ => todo!("other IO errors"),
        }
        .into()
    }
}

impl From<std::string::FromUtf8Error> for ErrorCode {
    fn from(_err: std::string::FromUtf8Error) -> ErrorCode {
        INVALID_DATA.into()
    }
}

pub type Result<T> = std::result::Result<T, ErrorCode>;

const CRATE_ERROR_BASE: u32 = 4096000;

/// Indicates a bug in this crate
pub const INTERNAL_ERROR: u32 = CRATE_ERROR_BASE;

/// Unexpected end of packet while unpacking
pub const NOT_ENOUGH_DATA: u32 = CRATE_ERROR_BASE + 1;
pub const CONNECTION_REFUSED: u32 = CRATE_ERROR_BASE + 2;
pub const CONNECTION_RESET: u32 = CRATE_ERROR_BASE + 3;
pub const HOST_UNREACHABLE: u32 = CRATE_ERROR_BASE + 4;
pub const NETWORK_UNREACHABLE: u32 = CRATE_ERROR_BASE + 5;
pub const CONNECTION_ABORTED: u32 = CRATE_ERROR_BASE + 5;
pub const NOT_CONNECTED: u32 = CRATE_ERROR_BASE + 6;
pub const INVALID_DATA: u32 = CRATE_ERROR_BASE + 7;
