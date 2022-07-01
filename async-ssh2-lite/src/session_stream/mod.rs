use core::{
    task::{Context, Poll},
    time::Duration,
};
use std::io::Error as IoError;

use async_trait::async_trait;
use ssh2::{BlockDirections, Error as Ssh2Error, Session};

use crate::error::Error;

//
#[cfg(feature = "async-io")]
mod impl_async_io;
#[cfg(feature = "tokio")]
mod impl_tokio;

//
#[async_trait]
pub trait AsyncSessionStream {
    //
    async fn x_with<R>(
        &self,
        op: impl FnMut() -> Result<R, Ssh2Error> + Send,
        sess: &Session,
        maybe_block_directions: BlockDirections,
        sleep_dur: Option<Duration>,
    ) -> Result<R, Error>;

    async fn read_and_write_with<R>(
        &self,
        op: impl FnMut() -> Result<R, Ssh2Error> + Send,
        sess: &Session,
    ) -> Result<R, Error> {
        self.x_with(
            op,
            sess,
            BlockDirections::Both,
            Some(Duration::from_millis(1)),
        )
        .await
    }

    async fn none_with<R>(
        &self,
        op: impl FnMut() -> Result<R, Ssh2Error> + Send,
        sess: &Session,
    ) -> Result<R, Error> {
        self.x_with(
            op,
            sess,
            BlockDirections::None,
            Some(Duration::from_millis(1)),
        )
        .await
    }

    async fn read_with<R>(
        &self,
        op: impl FnMut() -> Result<R, Ssh2Error> + Send,
        sess: &Session,
    ) -> Result<R, Error> {
        self.x_with(
            op,
            sess,
            BlockDirections::Inbound,
            Some(Duration::from_millis(1)),
        )
        .await
    }

    async fn write_with<R>(
        &self,
        op: impl FnMut() -> Result<R, Ssh2Error> + Send,
        sess: &Session,
    ) -> Result<R, Error> {
        self.x_with(
            op,
            sess,
            BlockDirections::Outbound,
            Some(Duration::from_millis(1)),
        )
        .await
    }

    //
    fn poll_x_with<R>(
        &self,
        cx: &mut Context,
        op: impl FnMut() -> Result<R, IoError> + Send,
        sess: &Session,
        maybe_block_directions: BlockDirections,
        sleep_dur: Option<Duration>,
    ) -> Poll<Result<R, IoError>>;

    fn poll_read_with<R>(
        &self,
        cx: &mut Context,
        op: impl FnMut() -> Result<R, IoError> + Send,
        sess: &Session,
    ) -> Poll<Result<R, IoError>> {
        self.poll_x_with(
            cx,
            op,
            sess,
            BlockDirections::Inbound,
            Some(Duration::from_millis(1)),
        )
    }

    fn poll_write_with<R>(
        &self,
        cx: &mut Context,
        op: impl FnMut() -> Result<R, IoError> + Send,
        sess: &Session,
    ) -> Poll<Result<R, IoError>> {
        self.poll_x_with(
            cx,
            op,
            sess,
            BlockDirections::Outbound,
            Some(Duration::from_millis(1)),
        )
    }
}

//
pub trait BlockDirectionsExt {
    fn is_readable(&self) -> bool;
    fn is_writable(&self) -> bool;
}
impl BlockDirectionsExt for BlockDirections {
    fn is_readable(&self) -> bool {
        matches!(self, BlockDirections::Inbound | BlockDirections::Both)
    }

    fn is_writable(&self) -> bool {
        matches!(self, BlockDirections::Outbound | BlockDirections::Both)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_directions_ext() {
        assert!(!BlockDirections::None.is_readable());
        assert!(!BlockDirections::None.is_writable());
        assert!(BlockDirections::Inbound.is_readable());
        assert!(!BlockDirections::Inbound.is_writable());
        assert!(!BlockDirections::Outbound.is_readable());
        assert!(BlockDirections::Outbound.is_writable());
        assert!(BlockDirections::Both.is_readable());
        assert!(BlockDirections::Both.is_writable());
    }
}
