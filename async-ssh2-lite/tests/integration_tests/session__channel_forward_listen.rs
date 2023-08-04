#![cfg(any(feature = "async-io", feature = "tokio"))]

use std::error;

use async_ssh2_lite::{ssh2::ErrorCode, AsyncSession, AsyncSessionStream, Error};
use futures_util::future::join_all;
#[cfg(not(feature = "_integration_tests_tokio_ext"))]
use futures_util::{AsyncReadExt as _, AsyncWriteExt as _};
#[cfg(feature = "_integration_tests_tokio_ext")]
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

use super::{
    helpers::get_connect_addr, session__userauth_pubkey::__run__session__userauth_pubkey_file,
};

//
#[cfg(feature = "tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 10)]
async fn simple_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    __run__session__userauth_pubkey_file(&mut session).await?;

    let mut client_sessions = vec![];
    for _ in 0..10 {
        let mut session =
            AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None)
                .await?;
        __run__session__userauth_pubkey_file(&mut session).await?;
        client_sessions.push(session);
    }

    __run__session__channel_forward_listen__with_tokio_spawn(session, client_sessions).await?;

    Ok(())
}

async fn __run__session__channel_forward_listen__with_tokio_spawn<
    S: AsyncSessionStream + Send + Sync + 'static,
>(
    session: AsyncSession<S>,
    client_sessions: Vec<AsyncSession<S>>,
) -> Result<(), Box<dyn error::Error>> {
    let mut remote_port = 1022;
    let mut n_retry = 0;
    let (mut listener, remote_port) = loop {
        #[allow(clippy::single_match)]
        match session
            .channel_forward_listen(remote_port, Some("127.0.0.1"), None)
            .await
        {
            Ok((listener, remote_port)) => break (listener, remote_port),
            Err(err) => {
                match &err {
                    Error::Ssh2(err) => match err.code() {
                        ErrorCode::Session(-32) => {
                            remote_port += 1;
                            continue;
                        }
                        _ => {}
                    },
                    _ => {}
                }

                n_retry += 1;
                if n_retry > 3 {
                    return Err(err.into());
                }

                return Err(err.into());
            }
        };
    };

    println!("run `netstat -tunlp | grep {remote_port}` in ssh server");
    println!("run `curl http://127.0.0.1:{remote_port}/ -v` in ssh server");

    //
    let server_task: tokio::task::JoinHandle<Result<(), Box<dyn error::Error + Send + Sync>>> =
        tokio::task::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok(mut channel) => {
                        tokio::task::spawn(async move {
                            let mut buf = vec![0; 128];
                            let mut n_read = 0;
                            let mut n_retry = 0;
                            loop {
                                let n = tokio::time::timeout(
                                    tokio::time::Duration::from_millis(3000),
                                    channel.read(&mut buf[n_read..]),
                                )
                                .await
                                .map_err(|err| {
                                    eprintln!("channel.read timeout");
                                    err
                                })??;
                                n_read += n;
                                if n == 0 {
                                    break;
                                }
                                // TODO, parse buf
                                if n_read >= 78 {
                                    break;
                                }
                                n_retry += 1;
                                if n_retry > 3 {
                                    eprintln!("Max read attempts reached.");
                                    break;
                                }
                            }
                            println!(
                                "channel.read successful, data:{}",
                                String::from_utf8_lossy(&buf)
                            );
                            if buf.starts_with(b"GET / HTTP/1.1\r\n") {
                                channel.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;
                            } else {
                                channel
                                    .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                                    .await?;
                            }
                            channel.send_eof().await?;
                            channel.wait_close().await?;

                            Result::<(), Box<dyn error::Error + Send + Sync>>::Ok(())
                        });
                    }
                    Err(err) => {
                        /*
                        Maybe
                        Ssh2(Error { code: Session(-23), msg: "Channel not found" })
                        */
                        eprintln!("listener.accept failed, err:{err:?}");
                        break Ok(());
                    }
                }
            }
        });

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    //
    let futures = client_sessions
        .into_iter()
        .enumerate()
        .map(|(i, session)| async move {
            let mut channel = session.channel_session().await?;
            channel
                .exec(
                    format!(r#"curl http://127.0.0.1:{remote_port}/ -v --retry 5 --retry-delay 0 -w "%{{http_code}}""#,)
                        .as_ref(),
                )
                .await?;
            let mut s = String::new();
            channel.read_to_string(&mut s).await?;
            println!("channel_forward_listen exec curl output:{s} i:{i}");
            channel.close().await?;
            println!(
                "channel_forward_listen exec curl exit_status:{} i:{i}",
                channel.exit_status()?
            );
            // TODO, https://github.com/bk-rs/ssh-rs/issues/17
            assert!(&["200".into(), "000".into()].contains(&s));
            Result::<_, Box<dyn error::Error>>::Ok(())
        })
        .collect::<Vec<_>>();

    let rets = join_all(futures).await;
    println!("channel_forward_listen exec curl rets:{rets:?}");
    assert!(rets.iter().all(|x| x.is_ok()));

    //
    server_task.abort();
    match server_task.await {
        Ok(_) => {}
        Err(err) => assert!(err.is_cancelled()),
    }

    Ok(())
}
