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
    /*
    if `for i in 0..12 {` Maybe
    Ssh2(Error { code: Session(-21), msg: "Channel open failure (connect failed)" })

    And the MaxSessions is 10.
    And sshd log has 'no more sessions'.
    */
    for i in 0..8 {
        let session = session.clone();
        let handle = tokio::spawn(async move {
            match __run__session__channel_session__exec(&session, i, "simple_with_tokio").await {
                Ok(_) => Ok(()),
                Err(err) => {
                    eprintln!(
                        "tokio_spawn_session simple_with_tokio __run__session__channel_session__exec i:{i} err:{err}"
                    );
                    Err(err.to_string())
                }
            }
        });
        handles.push(handle);
    }

    let mut rets = vec![];
    for handle in handles {
        rets.push(handle.await);
    }
    println!("tokio_spawn_session simple_with_tokio rets:{rets:?}");
    assert!(rets.iter().all(|x| x.as_ref().ok().unwrap().is_ok()));

    Ok(())
}

//
#[tokio::test]
async fn without_spawn_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    __run__session__userauth_pubkey_file(&mut session).await?;
    let session = Arc::new(session);

    let mut handles = vec![];

    for i in 0..20 {
        let session = session.clone();
        let handle = async move {
            match __run__session__channel_session__exec(&session, i, "without_spawn_with_tokio")
                .await
            {
                Ok(_) => Ok(()),
                Err(err) => {
                    eprintln!(
                        "tokio_spawn_session without_spawn_with_tokio __run__session__channel_session__exec i:{i} err:{err}"
                    );
                    Err(err.to_string())
                }
            }
        };
        handles.push(handle);
    }

    let mut rets = vec![];
    for handle in handles {
        rets.push(handle.await);
    }
    println!("tokio_spawn_session without_spawn_with_tokio rets:{rets:?}");
    assert!(rets.iter().all(|x| x.is_ok()));

    Ok(())
}

//
#[tokio::test(flavor = "multi_thread", worker_threads = 10)]
async fn concurrently_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    __run__session__userauth_pubkey_file(&mut session).await?;
    let session = Arc::new(session);

    let mut handles = vec![];
    /*
    if `for i in 0..3 {` Maybe
    Ssh2(Error { code: Session(-4), msg: "Unexpected error" })
    */
    for i in 0..1 {
        let session = session.clone();
        let handle = tokio::spawn(async move {
            match __run__session__channel_session__exec(&session, i, "concurrently_with_tokio")
                .await
            {
                Ok(_) => Ok(()),
                Err(err) => {
                    eprintln!(
                        "tokio_spawn_session concurrently_with_tokio __run__session__channel_session__exec i:{i} err:{err}"
                    );
                    Err(err.to_string())
                }
            }
        });
        handles.push(handle);
    }

    let rets = join_all(handles).await;
    println!("tokio_spawn_session concurrently_with_tokio rets:{rets:?}");
    assert!(rets.iter().all(|x| x.as_ref().ok().unwrap().is_ok()));

    Ok(())
}

async fn __run__session__channel_session__exec<S: AsyncSessionStream + Send + Sync + 'static>(
    session: &AsyncSession<S>,
    i: usize,
    case: &str,
) -> Result<(), Box<dyn error::Error>> {
    let mut channel = session.channel_session().await?;
    channel.exec("hostname").await?;
    let mut s = String::new();
    channel.read_to_string(&mut s).await?;
    println!("tokio_spawn_session {case} exec hostname output:{s} i:{i}");
    channel.close().await?;
    println!(
        "tokio_spawn_session {case} exec hostname exit_status:{} i:{i}",
        channel.exit_status()?
    );

    let mut channel = session.channel_session().await?;
    channel.exec("head -c 16354 /dev/random").await?;
    let mut b = vec![];
    channel.read_to_end(&mut b).await?;
    assert_eq!(b.len(), 16354);

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    channel.close().await?;
    println!(
        "tokio_spawn_session {case} exec head exit_status:{} i:{i}",
        channel.exit_status()?
    );

    Ok(())
}
