#![cfg(any(feature = "async-io", feature = "tokio"))]

use std::{
    error,
    path::{Path, PathBuf},
};

use async_ssh2_lite::{AsyncSession, AsyncSessionStream};
#[cfg(not(feature = "_integration_tests_tokio_ext"))]
use futures_util::{AsyncReadExt as _, AsyncWriteExt as _};
#[cfg(feature = "_integration_tests_tokio_ext")]
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use rand::{distributions::Alphanumeric, thread_rng, Rng as _};
use uuid::Uuid;

use super::{
    helpers::get_connect_addr, session__userauth_pubkey::__run__session__userauth_pubkey_file,
};

//
const FILE_SIZE: usize = 1024 * 512;

//
#[cfg(feature = "tokio")]
#[tokio::test]
async fn simple_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    __run__session__userauth_pubkey_file(&mut session).await?;

    let remote_path = PathBuf::from("/tmp").join(format!("scp_{}", Uuid::new_v4()));

    __run__session__scp_send(&mut session, &remote_path).await?;
    __run__session__scp_recv(&mut session, &remote_path).await?;

    Ok(())
}

#[cfg(feature = "async-io")]
#[test]
fn simple_with_async_io() -> Result<(), Box<dyn error::Error>> {
    futures_lite::future::block_on(async {
        let mut session =
            AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(get_connect_addr()?, None)
                .await?;
        __run__session__userauth_pubkey_file(&mut session).await?;

        let remote_path = PathBuf::from("/tmp").join(format!("scp_{}", Uuid::new_v4()));

        __run__session__scp_send(&mut session, &remote_path).await?;
        __run__session__scp_recv(&mut session, &remote_path).await?;

        Ok(())
    })
}

async fn __run__session__scp_send<S: AsyncSessionStream + Send + Sync + 'static>(
    session: &mut AsyncSession<S>,
    remote_path: &Path,
) -> Result<(), Box<dyn error::Error>> {
    let data: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(FILE_SIZE)
        .map(char::from)
        .collect();
    let data = data.as_bytes();

    let mut channel = session
        .scp_send(remote_path, 0o644, data.len() as u64, None)
        .await?;
    channel.write_all(data).await?;

    Ok(())
}

async fn __run__session__scp_recv<S: AsyncSessionStream + Send + Sync + 'static>(
    session: &mut AsyncSession<S>,
    remote_path: &Path,
) -> Result<(), Box<dyn error::Error>> {
    let (mut channel, stat) = session.scp_recv(remote_path).await?;
    println!(
        "scp_recv stat_size:{} stat_mode:{}",
        stat.size(),
        stat.mode()
    );
    assert_eq!(stat.size() as usize, FILE_SIZE);
    let mut contents = Vec::new();
    channel.read_to_end(&mut contents).await?;
    assert_eq!(contents.len(), stat.size() as usize);

    Ok(())
}
