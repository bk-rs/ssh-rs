pub use async_ssh2_lite;
pub use bb8;

#[cfg(feature = "tokio")]
mod impl_tokio;
#[cfg(feature = "tokio")]
pub use impl_tokio::AsyncSessionManagerWithTokioTcpStream;

use std::path::PathBuf;

//
#[derive(Debug, Clone)]
pub enum AsyncSessionUserauthType {
    Password {
        password: String,
    },
    Agent,
    PubkeyFile {
        pubkey: Option<PathBuf>,
        privatekey: PathBuf,
        passphrase: Option<String>,
    },
}

#[derive(Debug)]
pub enum AsyncSessionManagerError {
    ConnectError(async_ssh2_lite::Error),
    HandshakeError(async_ssh2_lite::Error),
    UserauthError(async_ssh2_lite::Error),
    AssertAuthenticated,
    Unknown(String),
}
impl core::fmt::Display for AsyncSessionManagerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for AsyncSessionManagerError {}
