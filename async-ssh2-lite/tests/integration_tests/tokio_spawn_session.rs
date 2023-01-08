#![cfg(feature = "tokio")]

use std::{error, sync::Arc};

use async_ssh2_lite::{AsyncSession, AsyncSessionStream};
use futures_util::future::join_all;
#[cfg(not(feature = "_integration_tests_tokio_ext"))]
use futures_util::AsyncReadExt as _;
#[cfg(feature = "_integration_tests_tokio_ext")]
use tokio::io::AsyncReadExt as _;

use super::{
    helpers::get_connect_addr, session__userauth_pubkey::__run__session__userauth_pubkey_file,
};

//
#[tokio::test]
async fn simple_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    __run__session__userauth_pubkey_file(&mut session).await?;
    let session = Arc::new(session);

    let mut handles = vec![];
    for i in 0..10 {
        let session = session.clone();
        let handle = tokio::spawn(async move {
            __run__session__channel_session__exec(&session, i)
                .await
                .unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}

//
#[tokio::test]
async fn concurrently_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    __run__session__userauth_pubkey_file(&mut session).await?;
    let session = Arc::new(session);

    let mut handles = vec![];
    for i in 0..10 {
        let session = session.clone();
        let handle = tokio::spawn(async move {
            __run__session__channel_session__exec(&session, i)
                .await
                .unwrap();
        });
        handles.push(handle);
    }

    let rets = join_all(handles).await;
    println!("tokio_spawn_session concurrently rets:{rets:?}");
    assert!(rets.iter().all(|x| x.is_ok()));

    Ok(())
}

async fn __run__session__channel_session__exec<S: AsyncSessionStream + Send + Sync + 'static>(
    session: &AsyncSession<S>,
    i: usize,
) -> Result<(), Box<dyn error::Error>> {
    let mut channel = session.channel_session().await?;
    channel.exec("hostname").await?;
    let mut s = String::new();
    channel.read_to_string(&mut s).await?;
    println!("tokio_spawn_session exec hostname output:{s} i:{i}");
    channel.close().await?;
    println!(
        "tokio_spawn_session exec hostname exit_status:{} i:{i}",
        channel.exit_status()?
    );

    let mut channel = session.channel_session().await?;
    channel.exec("head -c 16354 /dev/random").await?;
    let mut b = vec![];
    channel.read_to_end(&mut b).await?;
    assert_eq!(b.len(), 16354);
    channel.close().await?;
    println!(
        "tokio_spawn_session exec head exit_status:{} i:{i}",
        channel.exit_status()?
    );

    Ok(())
}
