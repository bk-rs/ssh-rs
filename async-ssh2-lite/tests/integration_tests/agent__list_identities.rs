#![cfg(any(feature = "async-io", feature = "tokio"))]

use std::error;

use async_ssh2_lite::{AsyncAgent, AsyncSession, AsyncSessionStream};

use super::{
    helpers::{get_connect_addr, is_internal_test_openssh_server},
    session__userauth_pubkey::__run__session__userauth_pubkey_file,
};

//
#[cfg(feature = "async-io")]
#[test]
fn simple_with_async_io() -> Result<(), Box<dyn error::Error>> {
    use async_ssh2_lite::async_io::Async;

    futures_lite::future::block_on(async {
        let stream = {
            cfg_if::cfg_if! {
                if #[cfg(unix)] {
                    use std::os::unix::net::UnixListener;
                    use tempfile::tempdir;

                    let dir = tempdir()?;
                    let path = dir.path().join("ssh_agent");
                    Async::<UnixListener>::bind(path)?
                } else {
                    use std::net::TcpListener;

                    Async::<TcpListener>::bind(([127, 0, 0, 1], 0))?
                }
            }
        };

        let mut agent = AsyncAgent::new(stream)?;
        __run__agent__list_identities(&mut agent).await?;

        Ok(())
    })
}

//
#[cfg(feature = "tokio")]
#[tokio::test]
async fn from_session_with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    __run__session__userauth_pubkey_file(&mut session).await?;
    __run__session__agent__list_identities(&session).await?;

    Ok(())
}

#[cfg(feature = "async-io")]
#[test]
fn from_session_with_async_io() -> Result<(), Box<dyn error::Error>> {
    futures_lite::future::block_on(async {
        let mut session =
            AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(get_connect_addr()?, None)
                .await?;
        __run__session__userauth_pubkey_file(&mut session).await?;
        __run__session__agent__list_identities(&session).await?;

        Ok(())
    })
}

async fn __run__session__agent__list_identities<S: AsyncSessionStream + Send + Sync + 'static>(
    session: &AsyncSession<S>,
) -> Result<(), Box<dyn error::Error>> {
    let mut agent = session.agent()?;
    __run__agent__list_identities(&mut agent).await?;

    Ok(())
}

async fn __run__agent__list_identities<S: AsyncSessionStream + Send + Sync + 'static>(
    agent: &mut AsyncAgent<S>,
) -> Result<(), Box<dyn error::Error>> {
    agent.connect().await?;

    agent.list_identities().await?;

    let identities = agent.identities()?;

    if is_internal_test_openssh_server() {
        assert!(identities
            .iter()
            .any(|x| x.comment().starts_with("ssh-rs/")))
    }

    for identity in identities {
        println!("identity comment:{}", identity.comment());
    }

    agent.disconnect().await?;

    Ok(())
}
