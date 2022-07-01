#![cfg(any(feature = "async-io", feature = "tokio"))]

use std::error;

use async_ssh2_lite::{AsyncSession, AsyncSessionStream};
use futures_util::future::join_all;

use super::helpers::{
    get_connect_addr, get_password, get_username, init_logger, is_internal_openssh_server_docker,
};

/*
Maybe LIBSSH2_ERROR_SOCKET_DISCONNECT , should change MaxStartups and MaxSessions
*/

//
#[cfg(feature = "tokio")]
#[tokio::test]
async fn simple_with_tokio() -> Result<(), Box<dyn error::Error>> {
    init_logger();

    let times = if is_internal_openssh_server_docker() {
        10
    } else {
        2
    };

    let futures = (1..=times)
        .into_iter()
        .map(|_| async {
            let mut session =
                AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None)
                    .await?;
            __run__session__userauth_password(&mut session).await?;
            Result::<_, Box<dyn error::Error>>::Ok(())
        })
        .collect::<Vec<_>>();

    let rets = join_all(futures).await;
    println!("__run__session__userauth_password rets:{:?}", rets);
    assert!(rets.iter().all(|x| x.is_ok()));

    Ok(())
}

#[cfg(feature = "async-io")]
#[test]
fn simple_with_async_io() -> Result<(), Box<dyn error::Error>> {
    futures_lite::future::block_on(async {
        init_logger();

        let times = if is_internal_openssh_server_docker() {
            10
        } else {
            2
        };

        let futures = (1..=times)
            .into_iter()
            .map(|_| async {
                let mut session = AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(
                    get_connect_addr()?,
                    None,
                )
                .await?;
                __run__session__userauth_password(&mut session).await?;
                Result::<_, Box<dyn error::Error>>::Ok(())
            })
            .collect::<Vec<_>>();

        let rets = join_all(futures).await;
        println!("__run__session__userauth_password rets:{:?}", rets);
        assert!(rets.iter().all(|x| x.is_ok()));

        Ok(())
    })
}

async fn __run__session__userauth_password<S: AsyncSessionStream + Send + Sync>(
    session: &mut AsyncSession<S>,
) -> Result<(), Box<dyn error::Error>> {
    session.handshake().await?;

    match session
        .userauth_password(get_username().as_ref(), "xxx")
        .await
    {
        Ok(_) => panic!(""),
        Err(err) => {
            assert!(err
                .to_string()
                .contains("Authentication failed (username/password)"));
        }
    }
    assert!(!session.authenticated());

    session
        .userauth_password(get_username().as_ref(), get_password().as_ref())
        .await?;
    assert!(session.authenticated());

    Ok(())
}
