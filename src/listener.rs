use std::io;
use std::sync::Arc;
use std::time::Duration;

use async_io::{Async, Timer};
use ssh2::Listener;

use crate::channel::AsyncChannel;

pub struct AsyncListener<S> {
    inner: Listener,
    async_io: Arc<Async<S>>,
}

impl<S> AsyncListener<S> {
    pub(crate) fn from_parts(inner: Listener, async_io: Arc<Async<S>>) -> Self {
        Self { inner, async_io }
    }
}

impl<S> AsyncListener<S> {
    pub async fn accept(&mut self) -> io::Result<AsyncChannel<S>> {
        // The I/O object for Listener::accept is on the remote SSH server. There is no way to poll
        // its state so the best we can do is loop and periodically check whether we have a new
        // connection.
        let channel = loop {
            match self.inner.accept() {
                Ok(channel) => break channel,
                Err(e)
                    if io::Error::from(ssh2::Error::from_errno(e.code())).kind()
                        == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(io::Error::from(e)),
            }

            Timer::after(Duration::from_millis(10)).await;
        };

        Ok(AsyncChannel::from_parts(channel, self.async_io.clone()))
    }
}
