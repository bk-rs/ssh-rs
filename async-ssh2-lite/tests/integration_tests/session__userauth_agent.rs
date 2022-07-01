#![cfg(any(feature = "async-io", feature = "tokio"))]

use std::error;

use async_ssh2_lite::{AsyncSession, AsyncSessionStream};

use super::helpers::{get_connect_addr, get_username};

//
#[cfg(feature = "tokio")]
#[tokio::test]
async fn simple_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    __run__session__userauth_agent_with_try_next(&mut session).await?;

    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    __run__session__userauth_agent(&mut session).await?;

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

        let mut session =
            AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(get_connect_addr()?, None)
                .await?;
        __run__session__userauth_agent(&mut session).await?;

        Ok(())
    })
}

pub(crate) async fn __run__session__userauth_agent_with_try_next<
    S: AsyncSessionStream + Send + Sync,
>(
    session: &mut AsyncSession<S>,
) -> Result<(), Box<dyn error::Error>> {
    session.handshake().await?;

    session
        .userauth_agent_with_try_next_with_callback(get_username().as_ref(), |identities| {
            identities.into_iter().rev().collect()
        })
        .await?;
    assert!(session.authenticated());

    Ok(())
}

async fn __run__session__userauth_agent<S: AsyncSessionStream + Send + Sync>(
    session: &mut AsyncSession<S>,
) -> Result<(), Box<dyn error::Error>> {
    session.handshake().await?;

    match session.userauth_agent(get_username().as_ref()).await {
        Ok(_) => {}
        Err(err) => {
            println!("session.userauth_agent failed, err:{}", err);
        }
    }
    assert!(!session.authenticated());

    Ok(())
}
