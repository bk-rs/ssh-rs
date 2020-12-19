use std::io;
use std::sync::Arc;

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawSocket, FromRawSocket};

use async_io::Async;
use ssh2::{Agent, PublicKey};

use crate::session::get_session;

pub struct AsyncAgent<S> {
    inner: Agent,
    async_io: Arc<Async<S>>,
}

#[cfg(unix)]
impl<S> AsyncAgent<S>
where
    S: AsRawFd + FromRawFd + 'static,
{
    pub fn new(stream: Async<S>) -> io::Result<Self> {
        let mut session = get_session(None)?;
        session.set_tcp_stream(stream.into_inner()?);

        let io = unsafe { S::from_raw_fd(session.as_raw_fd()) };
        let async_io = Arc::new(Async::new(io)?);

        let agent = session.agent()?;

        Ok(Self {
            inner: agent,
            async_io,
        })
    }
}

#[cfg(windows)]
impl<S> AsyncAgent<S>
where
    S: AsRawSocket + FromRawSocket + 'static,
{
    pub fn new(stream: Async<S>) -> io::Result<Self> {
        let mut session = get_session(None)?;
        session.set_tcp_stream(stream.into_inner()?);

        let io = unsafe { S::from_raw_socket(session.as_raw_socket()) };
        let async_io = Arc::new(Async::new(io)?);

        let agent = session.agent()?;

        Ok(Self {
            inner: agent,
            async_io,
        })
    }
}

impl<S> AsyncAgent<S> {
    pub(crate) fn from_parts(inner: Agent, async_io: Arc<Async<S>>) -> Self {
        Self { inner, async_io }
    }
}

impl<S> AsyncAgent<S> {
    pub async fn connect(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.connect().map_err(Into::into))
            .await
    }

    pub async fn disconnect(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.disconnect().map_err(Into::into))
            .await
    }

    pub async fn list_identities(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.list_identities().map_err(Into::into))
            .await
    }

    pub fn identities(&self) -> io::Result<Vec<PublicKey>> {
        self.inner.identities().map_err(Into::into)
    }

    pub async fn userauth(&self, username: &str, identity: &PublicKey) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.userauth(username, identity).map_err(Into::into))
            .await
    }
}
