/*
cargo run -p async-ssh2-lite-demo-smol --bin inspect_sftp 127.0.0.1:22 root
*/

use std::env;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;

use async_io::Async;
use futures::executor::block_on;
use uuid::Uuid;

use async_ssh2_lite::AsyncSession;

fn main() -> io::Result<()> {
    block_on(run())
}

async fn run() -> io::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| env::var("ADDR").unwrap_or("127.0.0.1:22".to_owned()));
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| env::var("USERNAME").unwrap_or("root".to_owned()));

    let addr = addr.to_socket_addrs().unwrap().next().unwrap();

    let stream = Async::<TcpStream>::connect(addr).await?;

    let mut session = AsyncSession::new(stream, None)?;

    session.handshake().await?;

    session.userauth_agent(username.as_ref()).await?;

    if !session.authenticated() {
        return Err(session
            .last_error()
            .and_then(|err| Some(io::Error::from(err)))
            .unwrap_or(io::Error::new(
                io::ErrorKind::Other,
                "unknown userauth error",
            )));
    }

    let sftp = session.sftp().await?;

    let filename = PathBuf::from("/tmp").join(Uuid::new_v4().to_string());
    let filename = filename.as_path();

    sftp.create(filename).await?;
    let file_stat = sftp.stat(filename).await?;
    println!("file_stat: {:?}", file_stat);

    sftp.unlink(filename).await?;

    println!("done");

    Ok(())
}
