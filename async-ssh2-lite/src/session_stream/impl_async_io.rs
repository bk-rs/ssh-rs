use core::{
    task::{Context, Poll},
    time::Duration,
};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use async_io::{Async, Timer};
use async_trait::async_trait;
use futures_util::future;
use futures_util::ready;
use ssh2::{BlockDirections, Error as Ssh2Error, Session};

use super::{AsyncSessionStream, BlockDirectionsExt as _};
use crate::{error::Error, util::ssh2_error_is_would_block};

//
#[async_trait]
impl<S> AsyncSessionStream for Async<S>
where
    S: Send + Sync,
{
    async fn x_with<R>(
        &self,
        mut op: impl FnMut() -> Result<R, Ssh2Error> + Send,
        sess: &Session,
        maybe_block_directions: BlockDirections,
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
                    assert!(maybe_block_directions.is_readable());

                    self.readable().await?
                }
                BlockDirections::Outbound => {
                    assert!(maybe_block_directions.is_writable());

                    self.writable().await?
                }
                BlockDirections::Both => {
                    assert!(maybe_block_directions.is_readable());
                    assert!(maybe_block_directions.is_writable());

                    let (ret, _) = future::select(self.readable(), self.writable())
                        .await
                        .factor_first();
                    ret?
                }
            }

            if let Some(dur) = sleep_dur {
                sleep(dur).await;
            }
        }
    }

    fn poll_x_with<R>(
        &self,
        cx: &mut Context,
        mut op: impl FnMut() -> Result<R, IoError> + Send,
        sess: &Session,
        maybe_block_directions: BlockDirections,
        _sleep_dur: Option<Duration>,
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
                assert!(maybe_block_directions.is_readable());

                ready!(self.poll_readable(cx))?;
            }
            BlockDirections::Outbound => {
                assert!(maybe_block_directions.is_writable());

                ready!(self.poll_writable(cx))?;
            }
            BlockDirections::Both => {
                assert!(maybe_block_directions.is_readable());
                assert!(maybe_block_directions.is_writable());

                ready!(self.poll_readable(cx))?;
                ready!(self.poll_writable(cx))?;
            }
        }

        Poll::Pending
    }
}

//
//
//
async fn sleep(dur: Duration) {
    Timer::after(dur).await;
}
