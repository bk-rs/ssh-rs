use std::error;

use async_ssh2_lite::{AsyncSession, AsyncSessionStream};

use super::helpers::{get_connect_addr, get_privatekey_path, get_username};

/*
id_rsa userauth_pubkey_file: Ssh2(Error { code: Session(-18), msg: "Username/PublicKey combination invalid" })
Ref https://github.com/libssh2/libssh2/issues/68
sudo tail -f /var/log/auth.log

id_dsa userauth_pubkey_memory: Ssh2(Error { code: Session(-19), msg: "Callback returned error" })
*/

//
#[cfg(feature = "tokio")]
#[tokio::test]
async fn simple_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    exec_userauth_pubkey_file(&mut session).await?;

    #[cfg(unix)]
    {
        let mut session =
            AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None)
                .await?;
        exec_userauth_pubkey_memory(&mut session).await?;
    }

    Ok(())
}

#[cfg(feature = "async-io")]
#[test]
fn simple_with_async_io() -> Result<(), Box<dyn error::Error>> {
    futures_lite::future::block_on(async {
        let mut session =
            AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(get_connect_addr()?, None)
                .await?;
        exec_userauth_pubkey_file(&mut session).await?;

        #[cfg(unix)]
        {
            let mut session = AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(
                get_connect_addr()?,
                None,
            )
            .await?;
            exec_userauth_pubkey_memory(&mut session).await?;
        }

        Ok(())
    })
}

async fn exec_userauth_pubkey_file<S: AsyncSessionStream + Send + Sync>(
    session: &mut AsyncSession<S>,
) -> Result<(), Box<dyn error::Error>> {
    session.handshake().await?;

    session
        .userauth_pubkey_file(
            get_username().as_ref(),
            None,
            get_privatekey_path().as_ref(),
            None,
        )
        .await?;
    assert!(session.authenticated());

    Ok(())
}

#[cfg(unix)]
async fn exec_userauth_pubkey_memory<S: AsyncSessionStream + Send + Sync>(
    session: &mut AsyncSession<S>,
) -> Result<(), Box<dyn error::Error>> {
    session.handshake().await?;

    session
        .userauth_pubkey_memory(
            get_username().as_ref(),
            None,
            String::from_utf8(std::fs::read(get_privatekey_path())?)?.as_ref(),
            None,
        )
        .await?;
    assert!(session.authenticated());

    Ok(())
}
