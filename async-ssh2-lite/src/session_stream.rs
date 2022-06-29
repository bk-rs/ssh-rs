use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use async_trait::async_trait;
use futures_util::future;
use ssh2::{BlockDirections, Error as Ssh2Error, Session};

use crate::error::Error;

//
#[async_trait]
pub trait AsyncSessionStream {
    async fn read_and_write_with<R>(
        &self,
        sess: &Session,
        op: impl FnMut() -> Result<R, Ssh2Error> + Send,
    ) -> Result<R, Error>;
}

//
#[cfg(feature = "async-io")]
#[async_trait]
impl<S> AsyncSessionStream for async_io::Async<S>
where
    S: Send + Sync,
{
    async fn read_and_write_with<R>(
        &self,
        sess: &Session,
        op: impl FnMut() -> Result<R, Ssh2Error> + Send,
    ) -> Result<R, Error> {
        let mut op = op;

        loop {
            match op() {
                Ok(x) => return Ok(x),
                Err(err) => {
                    if IoError::from(Ssh2Error::from_errno(err.code())).kind()
                        != IoErrorKind::WouldBlock
                    {
                        return Err(err.into());
                    }
                }
            }

            match sess.block_directions() {
                BlockDirections::None => {
                    unreachable!("")
                }
                BlockDirections::Inbound => self.readable().await?,
                BlockDirections::Outbound => self.writable().await?,
                BlockDirections::Both => {
                    let (ret, _) = future::select(self.readable(), self.writable())
                        .await
                        .factor_first();
                    ret?
                }
            }
        }
    }
}
