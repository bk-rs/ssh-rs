use std::io;
use std::sync::Arc;

use async_io::Async;
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
        let inner = &mut self.inner;

        let ret = self
            .async_io
            .write_with(|_| inner.accept().map_err(|err| err.into()))
            .await;

        ret.and_then(|channel| Ok(AsyncChannel::from_parts(channel, self.async_io.clone())))
    }
}
