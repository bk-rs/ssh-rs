use std::{env, error, path::PathBuf};

use async_ssh2_lite::{AsyncSession, AsyncSessionStream};

use super::helpers::{get_connect_addr, USERNAME};

/*
id_rsa userauth_pubkey_file: Ssh2(Error { code: Session(-18), msg: "Username/PublicKey combination invalid" })
Ref https://github.com/libssh2/libssh2/issues/68

id_dsa cannot userauth_pubkey_memory
*/

//
#[cfg(feature = "tokio")]
#[tokio::test]
async fn with_tokio() -> Result<(), Box<dyn error::Error>> {
    let mut session =
        AsyncSession::<async_ssh2_lite::TokioTcpStream>::connect(get_connect_addr()?, None).await?;
    exec_userauth_pubkey_file(&mut session).await?;

    Ok(())
}

#[cfg(feature = "async-io")]
#[test]
fn with_async_io() -> Result<(), Box<dyn error::Error>> {
    futures_lite::future::block_on(async {
        let mut session =
            AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(get_connect_addr()?, None)
                .await?;
        exec_userauth_pubkey_file(&mut session).await?;

        Ok(())
    })
}

async fn exec_userauth_pubkey_file<S: AsyncSessionStream + Send + Sync>(
    session: &mut AsyncSession<S>,
) -> Result<(), Box<dyn error::Error>> {
    session.handshake().await?;

    //
    let manifest_path = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(&manifest_dir)
    } else {
        PathBuf::new()
    };

    let keys_dir = manifest_path.join("tests").join("keys");
    let keys_dir = if keys_dir.exists() {
        keys_dir
    } else {
        manifest_path.join("tests").join("keys")
    };

    session
        .userauth_pubkey_file(USERNAME, None, &keys_dir.join("id_dsa"), None)
        .await?;
    assert!(session.authenticated());

    Ok(())
}
