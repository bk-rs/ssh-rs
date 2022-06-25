/*
cargo run -p async-ssh2-lite-demo-smol --bin download_file 127.0.0.1:22 root
*/

use std::env;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;

use async_io::Async;
use futures::executor::block_on;
use futures::AsyncReadExt;

use async_ssh2_lite::AsyncSession;

fn main() -> io::Result<()> {
    block_on(run())
}

async fn run() -> io::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:22".to_owned()));
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| env::var("USERNAME").unwrap_or_else(|_| "root".to_owned()));

    let addr = addr.to_socket_addrs().unwrap().next().unwrap();

    let stream = Async::<TcpStream>::connect(addr).await?;

    let mut session = AsyncSession::new(stream, None)?;

    session.handshake().await?;

    session.userauth_agent(username.as_ref()).await?;

    if !session.authenticated() {
        return Err(session
            .last_error()
            .map(io::Error::from)
            .unwrap_or_else(|| io::Error::new(io::ErrorKind::Other, "unknown userauth error")));
    }

    let mut channel = session.channel_session().await?;
    channel.exec("echo foo > /tmp/foo.txt").await?;
    channel.close().await?;
    println!("channel exit_status: {}", channel.exit_status()?);

    let (mut remote_file, stat) = session.scp_recv(Path::new("/tmp/foo.txt")).await?;
    println!("remote file size: {}", stat.size());
    let mut contents = Vec::new();
    remote_file.read_to_end(&mut contents).await?;
    assert_eq!(contents, b"foo\n");

    println!("done");

    Ok(())
}
