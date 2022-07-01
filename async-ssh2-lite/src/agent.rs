use std::sync::Arc;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;
#[cfg(windows)]
use std::os::windows::io::AsRawSocket;

use ssh2::{Agent, PublicKey, Session};

use crate::{error::Error, session::get_session, session_stream::AsyncSessionStream};

//
pub struct AsyncAgent<S> {
    inner: Agent,
    sess: Session,
    stream: Arc<S>,
}

#[cfg(unix)]
impl<S> AsyncAgent<S>
where
    S: AsRawFd + 'static,
{
    pub fn new(stream: S) -> Result<Self, Error> {
        let mut session = get_session(None)?;
        session.set_tcp_stream(stream.as_raw_fd());

        let stream = Arc::new(stream);

        let agent = session.agent()?;

        Ok(Self {
            inner: agent,
            sess: session,
            stream,
        })
    }
}

#[cfg(windows)]
impl<S> AsyncAgent<S>
where
    S: AsRawSocket + 'static,
{
    pub fn new(stream: S) -> Result<Self, Error> {
        let mut session = get_session(None)?;
        session.set_tcp_stream(stream.as_raw_socket());

        let stream = Arc::new(stream);

        let agent = session.agent()?;

        Ok(Self {
            inner: agent,
            sess: session,
            stream,
        })
    }
}

impl<S> AsyncAgent<S> {
    pub(crate) fn from_parts(inner: Agent, sess: Session, stream: Arc<S>) -> Self {
        Self {
            inner,
            sess,
            stream,
        }
    }
}

impl<S> AsyncAgent<S> {
    pub fn identities(&self) -> Result<Vec<PublicKey>, Error> {
        self.inner.identities().map_err(Into::into)
    }
}

impl<S> AsyncAgent<S>
where
    S: AsyncSessionStream + Send + Sync,
{
    pub async fn connect(&mut self) -> Result<(), Error> {
        self.stream
            .none_with(|| self.inner.connect(), &self.sess)
            .await
    }

    pub async fn disconnect(&mut self) -> Result<(), Error> {
        self.stream
            .none_with(|| self.inner.disconnect(), &self.sess)
            .await
    }

    pub async fn list_identities(&mut self) -> Result<(), Error> {
        self.stream
            .read_and_write_with(|| self.inner.list_identities(), &self.sess)
            .await
    }

    pub async fn userauth(&self, username: &str, identity: &PublicKey) -> Result<(), Error> {
        self.stream
            .read_and_write_with(|| self.inner.userauth(username, identity), &self.sess)
            .await
    }
}
