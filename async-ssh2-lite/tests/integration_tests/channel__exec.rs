#![cfg(any(feature = "async-io", feature = "tokio"))]

use std::error;

use async_ssh2_lite::{AsyncSession, AsyncSessionStream};
#[cfg(not(feature = "_integration_tests_tokio_ext"))]
use futures_util::AsyncReadExt as _;
#[cfg(feature = "_integration_tests_tokio_ext")]
use tokio::io::AsyncReadExt as _;

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
    __run__session__channel_session__exec(&mut session).await?;

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
        __run__session__channel_session__exec(&mut session).await?;

        Ok(())
    })
}

async fn __run__session__channel_session__exec<S: AsyncSessionStream + Send + Sync + 'static>(
    session: &mut AsyncSession<S>,
) -> Result<(), Box<dyn error::Error>> {
    let mut channel = session.channel_session().await?;
    channel.exec("hostname").await?;
    let mut s = String::new();
    channel.read_to_string(&mut s).await?;
    println!("exec hostname output:{s}");
    channel.close().await?;
    println!("exec hostname exit_status:{}", channel.exit_status()?);

    let mut channel = session.channel_session().await?;
    channel.exec("date").await?;
    let mut s = String::new();
    channel.read_to_string(&mut s).await?;
    println!("exec date output:{s}");
    channel.close().await?;
    println!("exec date exit_status:{}", channel.exit_status()?);

    let mut channel = session.channel_session().await?;
    channel.exec("head -c 16354 /dev/random").await?;
    let mut b = vec![];
    channel.read_to_end(&mut b).await?;
    assert_eq!(b.len(), 16354);
    channel.close().await?;
    println!("exec head exit_status:{}", channel.exit_status()?);

    Ok(())
}
