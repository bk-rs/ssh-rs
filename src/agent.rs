use std::io;
use std::sync::Arc;

use async_io::Async;
use ssh2::{Agent, PublicKey};

pub struct AsyncAgent<S> {
    inner: Agent,
    async_io: Arc<Async<S>>,
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
            .write_with(|_| inner.connect().map_err(|err| err.into()))
            .await
    }

    pub async fn disconnect(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.disconnect().map_err(|err| err.into()))
            .await
    }

    pub async fn list_identities(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.list_identities().map_err(|err| err.into()))
            .await
    }

    pub fn identities(&self) -> io::Result<Vec<PublicKey>> {
        self.inner.identities().map_err(|err| err.into())
    }

    pub async fn userauth(&self, username: &str, identity: &PublicKey) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.userauth(username, identity).map_err(|err| err.into()))
            .await
    }
}
