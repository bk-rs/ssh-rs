use core::{
    task::{Context, Poll},
    time::Duration,
};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use async_trait::async_trait;
use futures_util::ready;
use ssh2::{BlockDirections, Error as Ssh2Error, Session};
use tokio::net::TcpStream;
#[cfg(unix)]
use tokio::net::UnixStream;

use super::{AsyncSessionStream, BlockDirectionsExt as _};
use crate::{error::Error, util::ssh2_error_is_would_block};

//
#[async_trait]
impl AsyncSessionStream for TcpStream {
    async fn x_with<R>(
        &self,
        mut op: impl FnMut() -> Result<R, Ssh2Error> + Send,
        sess: &Session,
        expected_block_directions: BlockDirections,
        sleep_dur: Option<Duration>,
    ) -> Result<R, Error> {
        loop {
            match op() {
                Ok(x) => return Ok(x),
                Err(err) => {
                    if !ssh2_error_is_would_block(&err) {
                        return Err(err.into());
                    }
                }
            }

            match sess.block_directions() {
                BlockDirections::None => continue,
                BlockDirections::Inbound => {
                    assert!(expected_block_directions.is_readable());

                    self.readable().await?
                }
                BlockDirections::Outbound => {
                    assert!(expected_block_directions.is_writable());

                    self.writable().await?
                }
                BlockDirections::Both => {
                    assert!(expected_block_directions.is_readable());
                    assert!(expected_block_directions.is_writable());

                    self.ready(tokio::io::Interest::READABLE | tokio::io::Interest::WRITABLE)
                        .await?;
                }
            }

            if let Some(dur) = sleep_dur {
                sleep_async_fn(dur).await;
            }
        }
    }

    fn poll_x_with<R>(
        &self,
        cx: &mut Context,
        mut op: impl FnMut() -> Result<R, IoError> + Send,
        sess: &Session,
        expected_block_directions: BlockDirections,
        sleep_dur: Option<Duration>,
    ) -> Poll<Result<R, IoError>> {
        match op() {
            Err(err) if err.kind() == IoErrorKind::WouldBlock => {}
            ret => return Poll::Ready(ret),
        }

        match sess.block_directions() {
            BlockDirections::None => return Poll::Pending,
            BlockDirections::Inbound => {
                assert!(expected_block_directions.is_readable());

                ready!(self.poll_read_ready(cx))?;
            }
            BlockDirections::Outbound => {
                assert!(expected_block_directions.is_writable());

                ready!(self.poll_write_ready(cx))?;
            }
            BlockDirections::Both => {
                assert!(expected_block_directions.is_readable());
                assert!(expected_block_directions.is_writable());

                ready!(self.poll_write_ready(cx))?;
                ready!(self.poll_read_ready(cx))?;
            }
        }

        if let Some(dur) = sleep_dur {
            let waker = cx.waker().clone();
            tokio::spawn(async move {
                sleep_async_fn(dur).await;
                waker.wake();
            });
        } else {
            let waker = cx.waker().clone();
            waker.wake();
        }

        Poll::Pending
    }
}

#[cfg(unix)]
#[async_trait]
impl AsyncSessionStream for UnixStream {
    async fn x_with<R>(
        &self,
        mut op: impl FnMut() -> Result<R, Ssh2Error> + Send,
        sess: &Session,
        expected_block_directions: BlockDirections,
        sleep_dur: Option<Duration>,
    ) -> Result<R, Error> {
        loop {
            match op() {
                Ok(x) => return Ok(x),
                Err(err) => {
                    if !ssh2_error_is_would_block(&err) {
                        return Err(err.into());
                    }
                }
            }

            match sess.block_directions() {
                BlockDirections::None => {
                    unreachable!("")
                }
                BlockDirections::Inbound => {
                    assert!(expected_block_directions.is_readable());

                    self.readable().await?
                }
                BlockDirections::Outbound => {
                    assert!(expected_block_directions.is_writable());

                    self.writable().await?
                }
                BlockDirections::Both => {
                    assert!(expected_block_directions.is_readable());
                    assert!(expected_block_directions.is_writable());

                    self.ready(tokio::io::Interest::READABLE | tokio::io::Interest::WRITABLE)
                        .await?;
                }
            }

            if let Some(dur) = sleep_dur {
                sleep_async_fn(dur).await;
            }
        }
    }

    fn poll_x_with<R>(
        &self,
        cx: &mut Context,
        mut op: impl FnMut() -> Result<R, IoError> + Send,
        sess: &Session,
        expected_block_directions: BlockDirections,
        sleep_dur: Option<Duration>,
    ) -> Poll<Result<R, IoError>> {
        match op() {
            Err(err) if err.kind() == IoErrorKind::WouldBlock => {}
            ret => return Poll::Ready(ret),
        }

        match sess.block_directions() {
            BlockDirections::None => {
                unreachable!("")
            }
            BlockDirections::Inbound => {
                assert!(expected_block_directions.is_readable());

                ready!(self.poll_read_ready(cx))?;
            }
            BlockDirections::Outbound => {
                assert!(expected_block_directions.is_writable());

                ready!(self.poll_write_ready(cx))?;
            }
            BlockDirections::Both => {
                assert!(expected_block_directions.is_readable());
                assert!(expected_block_directions.is_writable());

                ready!(self.poll_write_ready(cx))?;
                ready!(self.poll_read_ready(cx))?;
            }
        }

        if let Some(dur) = sleep_dur {
            let waker = cx.waker().clone();
            tokio::spawn(async move {
                sleep_async_fn(dur).await;
                waker.wake();
            });
        } else {
            let waker = cx.waker().clone();
            waker.wake();
        }

        Poll::Pending
    }
}

//
//
//
async fn sleep_async_fn(dur: Duration) {
    sleep(dur).await;
}

fn sleep(dur: Duration) -> tokio::time::Sleep {
    tokio::time::sleep(tokio::time::Duration::from_millis(dur.as_millis() as u64))
}
