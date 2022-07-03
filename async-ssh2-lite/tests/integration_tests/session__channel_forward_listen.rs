#![cfg(any(feature = "async-io", feature = "tokio"))]

use std::error;

use async_ssh2_lite::{ssh2::ErrorCode, AsyncSession, AsyncSessionStream, Error};
use futures_util::future::join_all;
use futures_util::{AsyncReadExt as _, AsyncWriteExt as _};

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

    __run__session__channel_forward_listen__with_tokio_spawn(session).await?;

    Ok(())
}

#[cfg(feature = "async-io")]
#[tokio::test]
async fn simple_with_async_io() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(get_connect_addr()?, None)
            .await?;
    __run__session__userauth_pubkey_file(&mut session).await?;

    __run__session__channel_forward_listen__with_tokio_spawn(session).await?;

    Ok(())
}

async fn __run__session__channel_forward_listen__with_tokio_spawn<
    S: AsyncSessionStream + Send + Sync + 'static,
>(
    session: AsyncSession<S>,
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

    println!("run `netstat -tunlp | grep {}` in ssh server", remote_port);
    println!(
        "run `curl http://127.0.0.1:{}/ -v` in ssh server",
        remote_port
    );

    //
    let server_task: tokio::task::JoinHandle<Result<(), Box<dyn error::Error + Send + Sync>>> =
        tokio::task::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok(mut channel) => {
                        let mut buf = vec![0; 64];
                        channel.read(&mut buf).await?;
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
                            break;
                        }
                    }
                    Err(err) => {
                        eprintln!("listener.accept failed, err:{:?}", err);
                    }
                }
            }

            Ok(())
        });

    //
    let futures = (1..=10)
        .into_iter()
        .map(|i| {
            let session = session.clone();

            async move {
                let mut channel = session.channel_session().await?;
                channel
                    .exec(
                        format!(
                            r#"curl http://127.0.0.1:{}/ -v -w "%{{http_code}}""#,
                            remote_port
                        )
                        .as_ref(),
                    )
                    .await?;
                let mut s = String::new();
                channel.read_to_string(&mut s).await?;
                println!("exec curl output:{} i:{}", s, i);
                assert_eq!(s, "200");
                channel.close().await?;
                println!("exec curl exit_status:{} i:{}", channel.exit_status()?, i);
                Result::<_, Box<dyn error::Error>>::Ok(())
            }
        })
        .collect::<Vec<_>>();

    let rets = join_all(futures).await;
    println!("exec curl rets:{:?}", rets);
    assert!(rets.iter().all(|x| x.is_ok()));

    //
    server_task.abort();
    assert!(server_task.await.unwrap_err().is_cancelled());

    Ok(())
}
