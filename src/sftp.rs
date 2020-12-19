use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_io::Async;
use futures_util::io::{AsyncRead, AsyncWrite};
use futures_util::ready;
use ssh2::{File, FileStat, OpenFlags, OpenType, RenameFlags, Sftp};

pub struct AsyncSftp<S> {
    inner: Sftp,
    async_io: Arc<Async<S>>,
}

impl<S> AsyncSftp<S> {
    pub(crate) fn from_parts(inner: Sftp, async_io: Arc<Async<S>>) -> Self {
        Self { inner, async_io }
    }
}

impl<S> AsyncSftp<S> {
    pub async fn open_mode(
        &self,
        filename: &Path,
        flags: OpenFlags,
        mode: i32,
        open_type: OpenType,
    ) -> io::Result<AsyncFile<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| {
                inner
                    .open_mode(filename, flags, mode, open_type)
                    .map_err(Into::into)
            })
            .await;

        ret.map(|file| AsyncFile::from_parts(file, self.async_io.clone()))
    }

    pub async fn open(&self, filename: &Path) -> io::Result<AsyncFile<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| inner.open(filename).map_err(Into::into))
            .await;

        ret.map(|file| AsyncFile::from_parts(file, self.async_io.clone()))
    }

    pub async fn create(&self, filename: &Path) -> io::Result<AsyncFile<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| inner.create(filename).map_err(Into::into))
            .await;

        ret.map(|file| AsyncFile::from_parts(file, self.async_io.clone()))
    }

    pub async fn opendir(&self, dirname: &Path) -> io::Result<AsyncFile<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| inner.opendir(dirname).map_err(Into::into))
            .await;

        ret.map(|file| AsyncFile::from_parts(file, self.async_io.clone()))
    }

    pub async fn readdir(&self, dirname: &Path) -> io::Result<Vec<(PathBuf, FileStat)>> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.readdir(dirname).map_err(Into::into))
            .await
    }

    pub async fn mkdir(&self, filename: &Path, mode: i32) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.mkdir(filename, mode).map_err(Into::into))
            .await
    }

    pub async fn rmdir(&self, filename: &Path) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.rmdir(filename).map_err(Into::into))
            .await
    }

    pub async fn stat(&self, filename: &Path) -> io::Result<FileStat> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.stat(filename).map_err(Into::into))
            .await
    }

    pub async fn lstat(&self, filename: &Path) -> io::Result<FileStat> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.lstat(filename).map_err(Into::into))
            .await
    }

    pub async fn setstat(&self, filename: &Path, stat: FileStat) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.setstat(filename, stat.clone()).map_err(Into::into))
            .await
    }

    pub async fn symlink(&self, path: &Path, target: &Path) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.symlink(path, target).map_err(Into::into))
            .await
    }

    pub async fn readlink(&self, path: &Path) -> io::Result<PathBuf> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.readlink(path).map_err(Into::into))
            .await
    }

    pub async fn realpath(&self, path: &Path) -> io::Result<PathBuf> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.realpath(path).map_err(Into::into))
            .await
    }

    pub async fn rename(
        &self,
        src: &Path,
        dst: &Path,
        flags: Option<RenameFlags>,
    ) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.rename(src, dst, flags).map_err(Into::into))
            .await
    }

    pub async fn unlink(&self, file: &Path) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.unlink(file).map_err(Into::into))
            .await
    }
}

//
//
//
pub struct AsyncFile<S> {
    inner: File,
    async_io: Arc<Async<S>>,
}

impl<S> AsyncFile<S> {
    pub(crate) fn from_parts(inner: File, async_io: Arc<Async<S>>) -> Self {
        Self { inner, async_io }
    }
}

impl<S> AsyncRead for AsyncFile<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            match self.inner.read(buf) {
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                res => return Poll::Ready(res),
            }
            ready!(self.async_io.poll_readable(cx))?;
        }
    }
}

impl<S> AsyncWrite for AsyncFile<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            match self.inner.write(buf) {
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                res => return Poll::Ready(res),
            }
            ready!(self.async_io.poll_writable(cx))?;
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        loop {
            match self.inner.flush() {
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                res => return Poll::Ready(res),
            }
            ready!(self.async_io.poll_writable(cx))?;
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        self.poll_flush(cx)
    }
}
