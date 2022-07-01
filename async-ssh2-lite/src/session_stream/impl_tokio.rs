use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use async_trait::async_trait;
use ssh2::{BlockDirections, Error as Ssh2Error, Session};
use tokio::net::TcpStream;
#[cfg(unix)]
use tokio::net::UnixStream;

use super::AsyncSessionStream;
use crate::error::Error;

//
#[async_trait]
impl AsyncSessionStream for TcpStream {
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
                    self.ready(tokio::io::Interest::READABLE | tokio::io::Interest::WRITABLE)
                        .await?;
                }
            }
        }
    }
}

#[cfg(unix)]
#[async_trait]
impl AsyncSessionStream for UnixStream {
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
                    self.ready(tokio::io::Interest::READABLE | tokio::io::Interest::WRITABLE)
                        .await?;
                }
            }
        }
    }
}
