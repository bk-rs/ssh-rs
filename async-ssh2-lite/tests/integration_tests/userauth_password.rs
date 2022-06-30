use std::error;

use async_ssh2_lite::{AsyncSession, AsyncSessionStream};
use futures_util::future::join_all;

use super::helpers::{get_connect_addr, init_logger, PASSWORD, USERNAME};

/*
Maybe LIBSSH2_ERROR_SOCKET_DISCONNECT , should change MaxStartups and MaxSessions
*/

//
#[cfg(feature = "tokio")]
#[tokio::test]
async fn with_tokio() -> Result<(), Box<dyn error::Error>> {
    init_logger();

    let futures = (1_usize..=10)
        .into_iter()
        .map(|_| async {
            let session =
                AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None)
                    .await?;
            r#do(session).await?;
            Result::<_, Box<dyn error::Error>>::Ok(())
        })
        .collect::<Vec<_>>();

    let rets = join_all(futures).await;
    println!("{:?}", rets);
    assert!(rets.iter().all(|x| x.is_ok()));

    Ok(())
}

#[cfg(feature = "async-io")]
#[test]
fn with_async_io() -> Result<(), Box<dyn error::Error>> {
    futures_lite::future::block_on(async {
        init_logger();

        let futures = (1_usize..=10)
            .into_iter()
            .map(|_| async {
                let session = AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(
                    get_connect_addr()?,
                    None,
                )
                .await?;
                r#do(session).await?;
                Result::<_, Box<dyn error::Error>>::Ok(())
            })
            .collect::<Vec<_>>();

        let rets = join_all(futures).await;
        println!("{:?}", rets);
        assert!(rets.iter().all(|x| x.is_ok()));

        Ok(())
    })
}

async fn r#do<S: AsyncSessionStream + Send + Sync>(
    mut session: AsyncSession<S>,
) -> Result<(), Box<dyn error::Error>> {
    session.handshake().await?;

    match session.userauth_password(USERNAME, "xxx").await {
        Ok(_) => panic!(""),
        Err(err) => {
            assert!(err
                .to_string()
                .contains("Authentication failed (username/password)"));
        }
    }
    assert!(!session.authenticated());

    session.userauth_password(USERNAME, PASSWORD).await?;
    assert!(session.authenticated());

    Ok(())
}
