#![cfg(feature = "tokio")]

use std::{error, net::SocketAddr};

use async_ssh2_lite::{util::ConnectInfo, AsyncSession};
use futures_util::future::join_all;
#[cfg(not(feature = "_integration_tests_tokio_ext"))]
use futures_util::AsyncReadExt as _;
#[cfg(feature = "_integration_tests_tokio_ext")]
use tokio::io::AsyncReadExt as _;

use super::{
    helpers::{get_connect_addr, get_listen_addr, is_internal_test_openssh_server},
    session__userauth_pubkey::__run__session__userauth_pubkey_file,
};

#[tokio::test]
async fn simple_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let http_server_listen_addr = get_listen_addr();
    let http_server_listen_addr_for_server = http_server_listen_addr;
    let http_server_listen_addr_for_forwarding = SocketAddr::from((
        if is_internal_test_openssh_server() {
            [172, 17, 0, 1]
        } else {
            [127, 0, 0, 1]
        },
        http_server_listen_addr.port(),
    ));

    let ssh_server_connect_addr = get_connect_addr()?;

    let remote_port = portpicker::pick_unused_port().unwrap();

    //
    let server_task: tokio::task::JoinHandle<Result<(), Box<dyn error::Error + Send + Sync>>> =
        tokio::task::spawn(async move {
            use core::convert::Infallible;

            use hyper::{
                service::{make_service_fn, service_fn},
                Body, Request, Response, Server, StatusCode,
            };

            async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
                if req.uri().path() == "/200" {
                    Ok(Response::new(Body::from("")))
                } else {
                    let mut res = Response::new(Body::from(""));
                    *res.status_mut() = StatusCode::NOT_FOUND;
                    Ok(res)
                }
            }

            let addr = http_server_listen_addr_for_server;

            let make_service =
                make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

            let server = Server::bind(&addr).serve(make_service);

            match server.await {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("server error, err:{}", err);
                }
            }

            Ok(())
        });

    //
    let forwarding_task: tokio::task::JoinHandle<Result<(), Box<dyn error::Error + Send + Sync>>> =
        tokio::task::spawn(async move {
            let mut session = AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(
                ssh_server_connect_addr,
                None,
            )
            .await?;
            __run__session__userauth_pubkey_file(&mut session).await?;

            match session
                .remote_port_forwarding(
                    remote_port,
                    None,
                    None,
                    ConnectInfo::Tcp(http_server_listen_addr_for_forwarding),
                )
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("session.remote_port_forwarding error, err:{}", err);
                }
            }

            Ok(())
        });

    //
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(ssh_server_connect_addr, None)
            .await?;
    __run__session__userauth_pubkey_file(&mut session).await?;

    let futures = (1..=10)
        .into_iter()
        .map(|i| {
            let session = session.clone();

            async move {
                let mut channel = session.channel_session().await?;
                channel
                    .exec(
                        format!(
                            r#"curl http://127.0.0.1:{}/200 -H "x-foo: bar" -v -w "%{{http_code}}""#,
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

    forwarding_task.abort();
    assert!(forwarding_task.await.unwrap_err().is_cancelled());

    Ok(())
}
