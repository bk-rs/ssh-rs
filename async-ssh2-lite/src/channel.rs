use std::sync::Arc;

use ssh2::{Channel, ExitSignal, ExtendedData, PtyModes, ReadWindow, Session, Stream, WriteWindow};

use crate::{error::Error, session_stream::AsyncSessionStream};

//
pub struct AsyncChannel<S> {
    inner: Channel,
    sess: Session,
    stream: Arc<S>,
}

impl<S> AsyncChannel<S> {
    pub(crate) fn from_parts(inner: Channel, sess: Session, stream: Arc<S>) -> Self {
        Self {
            inner,
            sess,
            stream,
        }
    }
}

impl<S> AsyncChannel<S>
where
    S: AsyncSessionStream + Send + Sync + 'static,
{
    pub async fn setenv(&mut self, var: &str, val: &str) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.setenv(var, val), &self.sess)
            .await
    }

    pub async fn request_pty(
        &mut self,
        term: &str,
        mode: Option<PtyModes>,
        dim: Option<(u32, u32, u32, u32)>,
    ) -> Result<(), Error> {
        self.stream
            .rw_with(
                || self.inner.request_pty(term, mode.clone(), dim),
                &self.sess,
            )
            .await
    }

    pub async fn request_pty_size(
        &mut self,
        width: u32,
        height: u32,
        width_px: Option<u32>,
        height_px: Option<u32>,
    ) -> Result<(), Error> {
        self.stream
            .rw_with(
                || {
                    self.inner
                        .request_pty_size(width, height, width_px, height_px)
                },
                &self.sess,
            )
            .await
    }

    pub async fn request_auth_agent_forwarding(&mut self) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.request_auth_agent_forwarding(), &self.sess)
            .await
    }

    pub async fn exec(&mut self, command: &str) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.exec(command), &self.sess)
            .await
    }

    pub async fn shell(&mut self) -> Result<(), Error> {
        self.stream.rw_with(|| self.inner.shell(), &self.sess).await
    }

    pub async fn subsystem(&mut self, system: &str) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.subsystem(system), &self.sess)
            .await
    }

    pub async fn process_startup(
        &mut self,
        request: &str,
        message: Option<&str>,
    ) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.process_startup(request, message), &self.sess)
            .await
    }

    pub fn stderr(&self) -> AsyncStream<S> {
        AsyncStream::from_parts(self.inner.stderr(), self.sess.clone(), self.stream.clone())
    }

    pub fn stream(&self, stream_id: i32) -> AsyncStream<S> {
        AsyncStream::from_parts(
            self.inner.stream(stream_id),
            self.sess.clone(),
            self.stream.clone(),
        )
    }

    pub async fn handle_extended_data(&mut self, mode: ExtendedData) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.handle_extended_data(mode), &self.sess)
            .await
    }

    pub fn exit_status(&self) -> Result<i32, Error> {
        self.inner.exit_status().map_err(Into::into)
    }

    pub async fn exit_signal(&self) -> Result<ExitSignal, Error> {
        self.inner.exit_signal().map_err(Into::into)
    }

    pub fn read_window(&self) -> ReadWindow {
        self.inner.read_window()
    }
    pub fn write_window(&self) -> WriteWindow {
        self.inner.write_window()
    }

    pub async fn adjust_receive_window(&mut self, adjust: u64, force: bool) -> Result<u64, Error> {
        self.stream
            .rw_with(
                || self.inner.adjust_receive_window(adjust, force),
                &self.sess,
            )
            .await
    }

    pub fn eof(&self) -> bool {
        self.inner.eof()
    }

    pub async fn send_eof(&mut self) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.send_eof(), &self.sess)
            .await
    }

    pub async fn wait_eof(&mut self) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.wait_eof(), &self.sess)
            .await
    }

    pub async fn close(&mut self) -> Result<(), Error> {
        self.stream.rw_with(|| self.inner.close(), &self.sess).await
    }

    pub async fn wait_close(&mut self) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.wait_close(), &self.sess)
            .await
    }
}

//
pub struct AsyncStream<S> {
    inner: Stream,
    sess: Session,
    stream: Arc<S>,
}

impl<S> AsyncStream<S> {
    pub(crate) fn from_parts(inner: Stream, sess: Session, stream: Arc<S>) -> Self {
        Self {
            inner,
            sess,
            stream,
        }
    }
}

mod impl_futures_util {
    use core::{
        pin::Pin,
        task::{Context, Poll},
    };
    use std::io::{Error as IoError, Read as _, Write as _};

    use futures_util::io::{AsyncRead, AsyncWrite};

    use super::{AsyncChannel, AsyncStream};
    use crate::session_stream::AsyncSessionStream;

    //
    impl<S> AsyncRead for AsyncChannel<S>
    where
        S: AsyncSessionStream + Send + Sync + 'static,
    {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<Result<usize, IoError>> {
            Pin::new(&mut self.stream(0)).poll_read(cx, buf)
        }
    }

    impl<S> AsyncWrite for AsyncChannel<S>
    where
        S: AsyncSessionStream + Send + Sync + 'static,
    {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context,
            buf: &[u8],
        ) -> Poll<Result<usize, IoError>> {
            Pin::new(&mut self.stream(0)).poll_write(cx, buf)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), IoError>> {
            Pin::new(&mut self.stream(0)).poll_flush(cx)
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), IoError>> {
            Pin::new(&mut self.stream(0)).poll_close(cx)
        }
    }

    //
    impl<S> AsyncRead for AsyncStream<S>
    where
        S: AsyncSessionStream + Send + Sync + 'static,
    {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<Result<usize, IoError>> {
            let this = self.get_mut();
            let sess = this.sess.clone();
            let inner = &mut this.inner;

            this.stream.poll_read_with(cx, || inner.read(buf), &sess)
        }
    }

    impl<S> AsyncWrite for AsyncStream<S>
    where
        S: AsyncSessionStream + Send + Sync + 'static,
    {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context,
            buf: &[u8],
        ) -> Poll<Result<usize, IoError>> {
            let this = self.get_mut();
            let sess = this.sess.clone();
            let inner = &mut this.inner;

            this.stream.poll_write_with(cx, || inner.write(buf), &sess)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), IoError>> {
            let this = self.get_mut();
            let sess = this.sess.clone();
            let inner = &mut this.inner;

            this.stream.poll_write_with(cx, || inner.flush(), &sess)
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), IoError>> {
            self.poll_flush(cx)
        }
    }
}

#[cfg(feature = "tokio")]
mod impl_tokio {
    use core::{
        pin::Pin,
        task::{Context, Poll},
    };
    use std::io::{Error as IoError, Read as _, Write as _};

    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

    use super::{AsyncChannel, AsyncStream};
    use crate::session_stream::AsyncSessionStream;

    //
    impl<S> AsyncRead for AsyncChannel<S>
    where
        S: AsyncSessionStream + Send + Sync + 'static,
    {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<Result<(), IoError>> {
            Pin::new(&mut self.stream(0)).poll_read(cx, buf)
        }
    }

    impl<S> AsyncWrite for AsyncChannel<S>
    where
        S: AsyncSessionStream + Send + Sync + 'static,
    {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, IoError>> {
            Pin::new(&mut self.stream(0)).poll_write(cx, buf)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
            Pin::new(&mut self.stream(0)).poll_flush(cx)
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
            Pin::new(&mut self.stream(0)).poll_shutdown(cx)
        }
    }

    //
    impl<S> AsyncRead for AsyncStream<S>
    where
        S: AsyncSessionStream + Send + Sync + 'static,
    {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<Result<(), IoError>> {
            let this = self.get_mut();
            let sess = this.sess.clone();
            let inner = &mut this.inner;

            this.stream
                .poll_read_with(cx, || inner.read(buf.filled_mut()).map(|_| {}), &sess)
        }
    }

    impl<S> AsyncWrite for AsyncStream<S>
    where
        S: AsyncSessionStream + Send + Sync + 'static,
    {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, IoError>> {
            let this = self.get_mut();
            let sess = this.sess.clone();
            let inner = &mut this.inner;

            this.stream.poll_write_with(cx, || inner.write(buf), &sess)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
            let this = self.get_mut();
            let sess = this.sess.clone();
            let inner = &mut this.inner;

            this.stream.poll_write_with(cx, || inner.flush(), &sess)
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
            self.poll_flush(cx)
        }
    }
}
