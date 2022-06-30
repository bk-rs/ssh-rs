//! Asynchronous [ssh2](https://docs.rs/ssh2)

pub use ssh2;

#[cfg(feature = "async-io")]
pub use async_io;
#[cfg(feature = "async-io")]
pub type AsyncIoTcpStream = async_io::Async<std::net::TcpStream>;
#[cfg(all(unix, feature = "async-io"))]
pub type AsyncIoUnixStream = async_io::Async<std::os::unix::net::UnixStream>;

#[cfg(all(unix, feature = "tokio"))]
pub use tokio::net::UnixStream as TokioUnixStream;
#[cfg(feature = "tokio")]
pub use tokio::{self, net::TcpStream as TokioTcpStream};

//
pub mod agent;
pub mod channel;
pub mod listener;
pub mod session;
pub mod sftp;

pub use agent::AsyncAgent;
pub use channel::{AsyncChannel, AsyncStream};
pub use listener::AsyncListener;
pub use session::{AsyncSession, SessionConfiguration};
pub use sftp::{AsyncFile, AsyncSftp};

//
pub mod error;

pub use error::Error;

//
pub mod session_stream;

pub use session_stream::AsyncSessionStream;
