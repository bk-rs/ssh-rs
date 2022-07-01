use async_trait::async_trait;
use ssh2::{Error as Ssh2Error, Session};

use crate::error::Error;

//
#[cfg(feature = "async-io")]
mod impl_async_io;
#[cfg(feature = "tokio")]
mod impl_tokio;

//
#[async_trait]
pub trait AsyncSessionStream {
    async fn read_and_write_with<R>(
        &self,
        sess: &Session,
        op: impl FnMut() -> Result<R, Ssh2Error> + Send,
    ) -> Result<R, Error>;
}
