#![cfg(any(feature = "async-io", feature = "tokio"))]

use std::error;

use async_ssh2_lite::{AsyncSession, AsyncSessionStream};
use futures_util::AsyncReadExt as _;

use super::{
    helpers::get_connect_addr,
    session__userauth_agent::__run__session__userauth_agent_with_try_next,
};

//
#[cfg(feature = "tokio")]
#[tokio::test]
async fn simple_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    __run__session__userauth_agent_with_try_next(&mut session).await?;
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
        __run__session__userauth_agent_with_try_next(&mut session).await?;
        __run__session__channel_session__exec(&mut session).await?;

        Ok(())
    })
}

async fn __run__session__channel_session__exec<S: AsyncSessionStream + Send + Sync>(
    session: &mut AsyncSession<S>,
) -> Result<(), Box<dyn error::Error>> {
    let mut channel = session.channel_session().await?;
    channel.exec("hostname").await?;
    let mut s = String::new();
    channel.read_to_string(&mut s).await?;
    println!("hostname: {}", s);
    channel.close().await?;
    println!("channel exit_status: {}", channel.exit_status()?);

    let mut channel = session.channel_session().await?;
    channel.exec("date").await?;
    let mut s = String::new();
    channel.read_to_string(&mut s).await?;
    println!("date: {}", s);
    channel.close().await?;
    println!("channel exit_status: {}", channel.exit_status()?);

    Ok(())
}
