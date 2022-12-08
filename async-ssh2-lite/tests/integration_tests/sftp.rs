#![cfg(any(feature = "async-io", feature = "tokio"))]

use std::{error, path::PathBuf};

use async_ssh2_lite::{AsyncSession, AsyncSessionStream};
use uuid::Uuid;

use super::{
    helpers::get_connect_addr, session__userauth_pubkey::__run__session__userauth_pubkey_file,
};

//
#[cfg(feature = "tokio")]
#[tokio::test]
async fn simple_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    __run__session__userauth_pubkey_file(&mut session).await?;
    __run__session__sftp(&mut session).await?;

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
        __run__session__sftp(&mut session).await?;

        Ok(())
    })
}

async fn __run__session__sftp<S: AsyncSessionStream + Send + Sync + 'static>(
    session: &mut AsyncSession<S>,
) -> Result<(), Box<dyn error::Error>> {
    let sftp = session.sftp().await?;

    let remote_path = PathBuf::from("/tmp").join(format!("sftp_{}", Uuid::new_v4()));

    sftp.create(&remote_path).await?;

    let file_stat = sftp.stat(&remote_path).await?;
    println!("sftp file_stat:{file_stat:?}");

    let mut sftp_file = sftp.open(&remote_path).await?;
    let file_stat_for_file = sftp_file.stat().await?;
    println!("sftp file_stat_for_file:{file_stat_for_file:?}");
    sftp_file.close().await?;
    assert_eq!(file_stat, file_stat_for_file);

    sftp.unlink(&remote_path).await?;

    let list = sftp.readdir(&PathBuf::from("/")).await?;
    for (file_path, file_stat) in list.iter().take(10) {
        println!("sftp file_path:{file_path:?} file_stat:{file_stat:?}");
    }

    Ok(())
}
