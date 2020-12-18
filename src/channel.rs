use std::io::{self, Read, Write};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_io::Async;
use futures_util::future::FutureExt;
use futures_util::io::{AsyncRead, AsyncWrite};
use ssh2::{Channel, ExitSignal, ExtendedData, PtyModes, ReadWindow, Stream, WriteWindow};

pub struct AsyncChannel<S> {
    inner: Channel,
    async_io: Arc<Async<S>>,
}

impl<S> AsyncChannel<S> {
    pub(crate) fn from_parts(inner: Channel, async_io: Arc<Async<S>>) -> Self {
        Self { inner, async_io }
    }
}

impl<S> AsyncChannel<S> {
    pub async fn setenv(&mut self, var: &str, val: &str) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.setenv(var, val).map_err(|err| err.into()))
            .await
    }

    pub async fn request_pty(
        &mut self,
        term: &str,
        mode: Option<PtyModes>,
        dim: Option<(u32, u32, u32, u32)>,
    ) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .request_pty(term, mode.clone(), dim)
                    .map_err(|err| err.into())
            })
            .await
    }

    pub async fn request_pty_size(
        &mut self,
        width: u32,
        height: u32,
        width_px: Option<u32>,
        height_px: Option<u32>,
    ) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .request_pty_size(width, height, width_px, height_px)
                    .map_err(|err| err.into())
            })
            .await
    }

    pub async fn request_auth_agent_forwarding(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .request_auth_agent_forwarding()
                    .map_err(|err| err.into())
            })
            .await
    }

    pub async fn exec(&mut self, command: &str) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.exec(command).map_err(|err| err.into()))
            .await
    }

    pub async fn shell(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.shell().map_err(|err| err.into()))
            .await
    }

    pub async fn subsystem(&mut self, system: &str) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.subsystem(system).map_err(|err| err.into()))
            .await
    }

    pub async fn process_startup(
        &mut self,
        request: &str,
        message: Option<&str>,
    ) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .process_startup(request, message)
                    .map_err(|err| err.into())
            })
            .await
    }

    pub fn stderr(&self) -> AsyncStream<S> {
        AsyncStream::from_parts(self.inner.stderr(), self.async_io.clone())
    }

    pub fn stream(&self, stream_id: i32) -> AsyncStream<S> {
        AsyncStream::from_parts(self.inner.stream(stream_id), self.async_io.clone())
    }

    pub async fn handle_extended_data(&mut self, mode: ExtendedData) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.handle_extended_data(mode).map_err(|err| err.into()))
            .await
    }

    pub fn exit_status(&self) -> io::Result<i32> {
        self.inner.exit_status().map_err(|err| err.into())
    }

    pub async fn exit_signal(&self) -> io::Result<ExitSignal> {
        self.inner.exit_signal().map_err(|err| err.into())
    }

    pub fn read_window(&self) -> ReadWindow {
        self.inner.read_window()
    }
    pub fn write_window(&self) -> WriteWindow {
        self.inner.write_window()
    }

    pub async fn adjust_receive_window(&mut self, adjust: u64, force: bool) -> io::Result<u64> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .adjust_receive_window(adjust, force)
                    .map_err(|err| err.into())
            })
            .await
    }

    pub fn eof(&self) -> bool {
        self.inner.eof()
    }

    pub async fn send_eof(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.send_eof().map_err(|err| err.into()))
            .await
    }

    pub async fn wait_eof(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.wait_eof().map_err(|err| err.into()))
            .await
    }

    pub async fn close(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.close().map_err(|err| err.into()))
            .await
    }

    pub async fn wait_close(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.wait_close().map_err(|err| err.into()))
            .await
    }
}

impl<S> AsyncRead for AsyncChannel<S>
where
    S: Unpin + Send + Sync + 'static,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();

        Pin::new(&mut this.stream(0)).poll_read(cx, buf)
    }
}

impl<S> AsyncWrite for AsyncChannel<S>
where
    S: Unpin + Send + Sync + 'static,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
        let this = self.get_mut();

        Pin::new(&mut this.stream(0)).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        let this = self.get_mut();

        Pin::new(&mut this.stream(0)).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        let this = self.get_mut();

        Pin::new(&mut this.stream(0)).poll_close(cx)
    }
}

//
//
//
pub struct AsyncStream<S> {
    inner: Stream,
    async_io: Arc<Async<S>>,
}

impl<S> AsyncStream<S> {
    pub(crate) fn from_parts(inner: Stream, async_io: Arc<Async<S>>) -> Self {
        Self { inner, async_io }
    }
}

impl<S> AsyncRead for AsyncStream<S>
where
    S: Unpin + Send + Sync + 'static,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.async_io
            .clone()
            .read_with(|_| self.inner.read(buf))
            .boxed()
            .poll_unpin(cx)
    }
}

impl<S> AsyncWrite for AsyncStream<S>
where
    S: Unpin + Send + Sync + 'static,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.async_io
            .clone()
            .write_with(|_| self.inner.write(buf))
            .boxed()
            .poll_unpin(cx)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        self.async_io
            .clone()
            .write_with(|_| self.inner.flush())
            .boxed()
            .poll_unpin(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        self.async_io
            .clone()
            .write_with(|_| {
                // TODO
                Ok(())
            })
            .boxed()
            .poll_unpin(cx)
    }
}
