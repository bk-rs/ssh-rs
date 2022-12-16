use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind},
    net::SocketAddr,
    path::Path,
};

use ssh2::Error as Ssh2Error;

//
pub fn ssh2_error_is_would_block(err: &Ssh2Error) -> bool {
    IoError::from(Ssh2Error::from_errno(err.code())).kind() == IoErrorKind::WouldBlock
}

//
#[derive(Debug, Clone)]
pub enum ConnectInfo {
    Tcp(SocketAddr),
    #[cfg(unix)]
    Unix(Box<Path>),
}

impl ConnectInfo {
    pub fn with_tcp(addr: impl Into<SocketAddr>) -> Self {
        Self::Tcp(addr.into())
    }

    #[cfg(unix)]
    pub fn with_unix(path: impl AsRef<Path>) -> Self {
        Self::Unix(path.as_ref().into())
    }
}

#[cfg(windows)]
pub struct RawSocketWrapper(pub std::os::windows::io::RawSocket);

#[cfg(windows)]
impl std::os::windows::io::AsRawSocket for RawSocketWrapper {
    #[inline]
    fn as_raw_socket(&self) -> std::os::windows::io::RawSocket {
        self.0
    }
}
