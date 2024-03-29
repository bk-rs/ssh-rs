/*
RUST_BACKTRACE=1 RUST_LOG=trace cargo run -p bb8-async-ssh2-lite-demo --bin bb8_asl_demo_tokio_tcp_stream -- 127.0.0.1:22 root '~/.ssh/id_rsa'
*/

use std::{env, path::PathBuf};

use bb8_async_ssh2_lite::{
    bb8, AsyncSessionManagerWithTokioTcpStream, AsyncSessionUserauthType,
    AsyncSftpManagerWithTokioTcpStream,
};
use futures_util::{future::join_all, AsyncReadExt as _};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_addr = env::args().nth(1).ok_or("socket_addr missing")?.parse()?;
    let username = env::args().nth(2).ok_or("username missing")?;
    let privatekey: PathBuf = env::args().nth(3).ok_or("privatekey missing")?.parse()?;

    {
        let mut mgr = AsyncSessionManagerWithTokioTcpStream::new(
            socket_addr,
            None,
            &username,
            AsyncSessionUserauthType::PubkeyFile {
                pubkey: None,
                privatekey: privatekey.to_owned(),
                passphrase: None,
            },
        );
        mgr.set_max_number_of_unauthenticated_conns(10);

        let pool = bb8::Pool::builder().max_size(20).build(mgr).await?;

        let mut handles = vec![];
        for i in 0..100 {
            let pool = pool.clone();
            let handle = tokio::spawn(async move {
                let session = pool.get().await?;

                let mut channel = session.channel_session().await?;
                channel.exec("hostname").await?;
                let mut s = String::new();
                channel.read_to_string(&mut s).await?;
                println!("exec hostname output:{s} i:{i}");
                channel.close().await?;
                println!("exec hostname exit_status:{} i:{i}", channel.exit_status()?);

                Result::<(), Box<dyn std::error::Error + Send + Sync>>::Ok(())
            });
            handles.push(handle);
        }

        let rets = join_all(handles).await;
        println!("rets:{rets:?}");
        assert!(rets.iter().all(|x| x.as_ref().ok().unwrap().is_ok()));
    }

    {
        let mut mgr = AsyncSessionManagerWithTokioTcpStream::new(
            socket_addr,
            None,
            &username,
            AsyncSessionUserauthType::PubkeyFile {
                pubkey: None,
                privatekey,
                passphrase: None,
            },
        );
        mgr.set_max_number_of_unauthenticated_conns(10);

        let mgr = AsyncSftpManagerWithTokioTcpStream(mgr);

        let pool = bb8::Pool::builder().max_size(20).build(mgr).await?;

        let mut handles = vec![];
        for i in 0..100 {
            let pool = pool.clone();
            let handle = tokio::spawn(async move {
                let sftp = pool.get().await?;

                let list = sftp.readdir(&PathBuf::from("/tmp")).await?;
                println!("readdir /tmp list:{} i:{i}", list.len());

                Result::<(), Box<dyn std::error::Error + Send + Sync>>::Ok(())
            });
            handles.push(handle);
        }

        let rets = join_all(handles).await;
        println!("rets:{rets:?}");
        assert!(rets.iter().all(|x| x.as_ref().ok().unwrap().is_ok()));
    }

    Ok(())
}
