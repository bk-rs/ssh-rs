use core::time::Duration;
use std::sync::Arc;

use ssh2::{BlockDirections, Listener, Session};

use crate::{channel::AsyncChannel, error::Error, session_stream::AsyncSessionStream};

//
pub struct AsyncListener<S> {
    inner: Listener,
    sess: Session,
    stream: Arc<S>,
}

impl<S> AsyncListener<S> {
    pub(crate) fn from_parts(inner: Listener, sess: Session, stream: Arc<S>) -> Self {
        Self {
            inner,
            sess,
            stream,
        }
    }
}

impl<S> AsyncListener<S>
where
    S: AsyncSessionStream + Send + Sync,
{
    pub async fn accept(&mut self) -> Result<AsyncChannel<S>, Error> {
        let channel = self
            .stream
            .x_with(
                || self.inner.accept(),
                &self.sess,
                BlockDirections::Both,
                Some(Duration::from_millis(10)),
            )
            .await?;

        Ok(AsyncChannel::from_parts(
            channel,
            self.sess.clone(),
            self.stream.clone(),
        ))
    }
}
