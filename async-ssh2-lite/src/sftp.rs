use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use ssh2::{File, FileStat, OpenFlags, OpenType, RenameFlags, Session, Sftp};

use crate::{error::Error, session_stream::AsyncSessionStream};

//
pub struct AsyncSftp<S> {
    inner: Sftp,
    sess: Session,
    stream: Arc<S>,
}

impl<S> AsyncSftp<S> {
    pub(crate) fn from_parts(inner: Sftp, sess: Session, stream: Arc<S>) -> Self {
        Self {
            inner,
            sess,
            stream,
        }
    }
}

impl<S> AsyncSftp<S>
where
    S: AsyncSessionStream + Send + Sync + 'static,
{
    pub async fn open_mode(
        &self,
        filename: &Path,
        flags: OpenFlags,
        mode: i32,
        open_type: OpenType,
    ) -> Result<AsyncFile<S>, Error> {
        let file = self
            .stream
            .rw_with(
                || self.inner.open_mode(filename, flags, mode, open_type),
                &self.sess,
            )
            .await?;

        Ok(AsyncFile::from_parts(
            file,
            self.sess.clone(),
            self.stream.clone(),
        ))
    }

    pub async fn open(&self, filename: &Path) -> Result<AsyncFile<S>, Error> {
        let file = self
            .stream
            .rw_with(|| self.inner.open(filename), &self.sess)
            .await?;

        Ok(AsyncFile::from_parts(
            file,
            self.sess.clone(),
            self.stream.clone(),
        ))
    }

    pub async fn create(&self, filename: &Path) -> Result<AsyncFile<S>, Error> {
        let file = self
            .stream
            .rw_with(|| self.inner.create(filename), &self.sess)
            .await?;

        Ok(AsyncFile::from_parts(
            file,
            self.sess.clone(),
            self.stream.clone(),
        ))
    }

    pub async fn opendir(&self, dirname: &Path) -> Result<AsyncFile<S>, Error> {
        let file = self
            .stream
            .rw_with(|| self.inner.opendir(dirname), &self.sess)
            .await?;

        Ok(AsyncFile::from_parts(
            file,
            self.sess.clone(),
            self.stream.clone(),
        ))
    }

    pub async fn readdir(&self, dirname: &Path) -> Result<Vec<(PathBuf, FileStat)>, Error> {
        // Copy from ssh2
        let mut dir = self.opendir(dirname).await?;
        let mut ret = Vec::new();
        loop {
            match dir.readdir().await {
                Ok((filename, stat)) => {
                    if &*filename == Path::new(".") || &*filename == Path::new("..") {
                        continue;
                    }

                    ret.push((dirname.join(&filename), stat))
                }
                Err(Error::Ssh2(ref e))
                    if e.code() == ssh2::ErrorCode::Session(libssh2_sys::LIBSSH2_ERROR_FILE) =>
                {
                    break
                }
                Err(e) => return Err(e),
            }
        }
        Ok(ret)
    }

    pub async fn mkdir(&self, filename: &Path, mode: i32) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.mkdir(filename, mode), &self.sess)
            .await
    }

    pub async fn rmdir(&self, filename: &Path) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.rmdir(filename), &self.sess)
            .await
    }

    pub async fn stat(&self, filename: &Path) -> Result<FileStat, Error> {
        self.stream
            .rw_with(|| self.inner.stat(filename), &self.sess)
            .await
    }

    pub async fn lstat(&self, filename: &Path) -> Result<FileStat, Error> {
        self.stream
            .rw_with(|| self.inner.lstat(filename), &self.sess)
            .await
    }

    pub async fn setstat(&self, filename: &Path, stat: FileStat) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.setstat(filename, stat.clone()), &self.sess)
            .await
    }

    pub async fn symlink(&self, path: &Path, target: &Path) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.symlink(path, target), &self.sess)
            .await
    }

    pub async fn readlink(&self, path: &Path) -> Result<PathBuf, Error> {
        self.stream
            .rw_with(|| self.inner.readlink(path), &self.sess)
            .await
    }

    pub async fn realpath(&self, path: &Path) -> Result<PathBuf, Error> {
        self.stream
            .rw_with(|| self.inner.realpath(path), &self.sess)
            .await
    }

    pub async fn rename(
        &self,
        src: &Path,
        dst: &Path,
        flags: Option<RenameFlags>,
    ) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.rename(src, dst, flags), &self.sess)
            .await
    }

    pub async fn unlink(&self, file: &Path) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.unlink(file), &self.sess)
            .await
    }

    pub async fn shutdown(&mut self) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.shutdown(), &self.sess)
            .await
    }
}

//
pub struct AsyncFile<S> {
    inner: File,
    sess: Session,
    stream: Arc<S>,
}

impl<S> AsyncFile<S> {
    pub(crate) fn from_parts(inner: File, sess: Session, stream: Arc<S>) -> Self {
        Self {
            inner,
            sess,
            stream,
        }
    }
}

impl<S> AsyncFile<S>
where
    S: AsyncSessionStream + Send + Sync + 'static,
{
    pub async fn readdir(&mut self) -> Result<(PathBuf, FileStat), Error> {
        self.stream
            .rw_with(|| self.inner.readdir(), &self.sess)
            .await
    }
}

mod impl_futures_util {
    use core::{
        pin::Pin,
        task::{Context, Poll},
    };
    use std::io::{Error as IoError, Read as _, Seek, SeekFrom, Write as _};

    use futures_util::io::{AsyncRead, AsyncSeek, AsyncWrite};

    use super::AsyncFile;
    use crate::session_stream::AsyncSessionStream;

    //
    impl<S> AsyncRead for AsyncFile<S>
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

    impl<S> AsyncWrite for AsyncFile<S>
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

    impl<S> AsyncSeek for AsyncFile<S>
    where
        S: AsyncSessionStream + Send + Sync + 'static,
    {
        fn poll_seek(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            pos: SeekFrom,
        ) -> Poll<Result<u64, IoError>> {
            let this = self.get_mut();
            let sess = this.sess.clone();
            let inner = &mut this.inner;

            this.stream.poll_read_with(cx, || inner.seek(pos), &sess)
        }
    }
}
