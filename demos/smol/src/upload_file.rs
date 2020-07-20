/*
cargo run -p async-ssh2-lite-demo-smol --bin upload_file 127.0.0.1:22 root
*/

use std::env;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;

use async_io::Async;
use blocking::block_on;
use futures::AsyncWriteExt;

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

    let mut remote_file = session
        .scp_send(Path::new("/tmp/bar.txt"), 0o644, 10, None)
        .await?;
    remote_file.write(b"1234567890").await?;

    println!("done");

    Ok(())
}
