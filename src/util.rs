use std::future::Future;
use std::task::{Context, Poll};

// ref https://github.com/stjepang/async-io/blob/v0.1.2/src/lib.rs#L1262-L1266
pub(crate) fn poll_once<T>(cx: &mut Context<'_>, fut: impl Future<Output = T>) -> Poll<T> {
    pin_utils::pin_mut!(fut);
    fut.poll(cx)
}
