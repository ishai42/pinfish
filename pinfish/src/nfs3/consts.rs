// RFC1813 defines the status codes as enum
pub const NFS3_OK: u32 = 0;
pub const NFS3ERR_PERM: u32 = 1;
pub const NFS3ERR_NOENT: u32 = 2;
pub const NFS3ERR_IO: u32 = 5;
pub const NFS3ERR_NXIO: u32 = 6;
pub const NFS3ERR_ACCES: u32 = 13;
pub const NFS3ERR_EXIST: u32 = 17;
pub const NFS3ERR_XDEV: u32 = 18;
pub const NFS3ERR_NODEV: u32 = 19;
pub const NFS3ERR_NOTDIR: u32 = 20;
pub const NFS3ERR_ISDIR: u32 = 21;
pub const NFS3ERR_INVAL: u32 = 22;
pub const NFS3ERR_FBIG: u32 = 27;
pub const NFS3ERR_NOSPC: u32 = 28;
pub const NFS3ERR_ROFS: u32 = 30;
pub const NFS3ERR_MLINK: u32 = 31;
pub const NFS3ERR_NAMETOOLONG: u32 = 63;
pub const NFS3ERR_NOTEMPTY: u32 = 66;
pub const NFS3ERR_DQUOT: u32 = 69;
pub const NFS3ERR_STALE: u32 = 70;
pub const NFS3ERR_REMOTE: u32 = 71;
pub const NFS3ERR_BADHANDLE: u32 = 10001;
pub const NFS3ERR_NOT_SYNC: u32 = 10002;
pub const NFS3ERR_BAD_COOKIE: u32 = 10003;
pub const NFS3ERR_NOTSUPP: u32 = 10004;
pub const NFS3ERR_TOOSMALL: u32 = 10005;
pub const NFS3ERR_SERVERFAULT: u32 = 10006;
pub const NFS3ERR_BADTYPE: u32 = 10007;
pub const NFS3ERR_JUKEBOX: u32 = 10008;

pub const NFSPROC3_NULL: u32 = 0;
pub const NFSPROC3_GETATTR: u32 = 1;
pub const NFSPROC3_SETATTR: u32 = 2;
pub const NFSPROC3_LOOKUP: u32 = 3;
pub const NFSPROC3_ACCESS: u32 = 4;
pub const NFSPROC3_READLINK: u32 = 5;
pub const NFSPROC3_READ: u32 = 6;
pub const NFSPROC3_WRITE: u32 = 7;
pub const NFSPROC3_CREATE: u32 = 8;
pub const NFSPROC3_MKDIR: u32 = 9;
pub const NFSPROC3_SYMLINK: u32 = 10;
pub const NFSPROC3_MKNOD: u32 = 11;
pub const NFSPROC3_REMOVE: u32 = 12;
pub const NFSPROC3_RMDIR: u32 = 13;
pub const NFSPROC3_RENAME: u32 = 14;
pub const NFSPROC3_LINK: u32 = 15;
pub const NFSPROC3_READDIR: u32 = 16;
pub const NFSPROC3_READDIRPLUS: u32 = 17;
pub const NFSPROC3_FSTAT: u32 = 18;
pub const NFSPROC3_FSINFO: u32 = 19;
pub const NFSPROC3_PATCHCONF: u32 = 20;
pub const NFSPROC3_COMMIT: u32 = 21;
