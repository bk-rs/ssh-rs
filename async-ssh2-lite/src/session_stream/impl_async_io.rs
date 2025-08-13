use core::{
    task::{Context, Poll},
    time::Duration,
};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use async_io::{Async, Timer};
use async_trait::async_trait;
use futures_util::{future, pin_mut, ready};
use ssh2::{BlockDirections, Error as Ssh2Error, Session};

use super::AsyncSessionStream;
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
        _expected_block_directions: BlockDirections,
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

            // Wait for whatever I/O the session needs, not what we expect.
            // The session knows the aggregate state of all channels.
            match sess.block_directions() {
                BlockDirections::None => continue,
                BlockDirections::Inbound => {
                    // Session needs to read data (could be for any channel)
                    self.readable().await?
                }
                BlockDirections::Outbound => {
                    // Session needs to write data (could be for any channel)
                    self.writable().await?
                }
                BlockDirections::Both => {
                    // Session needs both read and write
                    let (ret, _) = future::select(self.readable(), self.writable())
                        .await
                        .factor_first();
                    ret?
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
        _expected_block_directions: BlockDirections,
        sleep_dur: Option<Duration>,
    ) -> Poll<Result<R, IoError>> {
        match op() {
            Err(err) if err.kind() == IoErrorKind::WouldBlock => {}
            ret => return Poll::Ready(ret),
        }

        // Wait for whatever I/O the session needs, not what we expect.
        // The session knows the aggregate state of all channels.
        match sess.block_directions() {
            BlockDirections::None => return Poll::Pending,
            BlockDirections::Inbound => {
                // Session needs to read data (could be for any channel)
                ready!(self.poll_readable(cx))?;
            }
            BlockDirections::Outbound => {
                // Session needs to write data (could be for any channel)
                ready!(self.poll_writable(cx))?;
            }
            BlockDirections::Both => {
                // Session needs both read and write
                // Must first poll_writable, because session__scp_send_and_scp_recv.rs
                ready!(self.poll_writable(cx))?;
                ready!(self.poll_readable(cx))?;
            }
        }

        if let Some(dur) = sleep_dur {
            let waker = cx.waker().clone();
            // TODO, maybe wrong
            let timer = sleep(dur);
            pin_mut!(timer);
            ready!(future::Future::poll(timer, cx));
            waker.wake();
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

async fn sleep(dur: Duration) -> Timer {
    Timer::after(dur)
}
