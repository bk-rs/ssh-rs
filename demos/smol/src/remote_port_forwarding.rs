/*
cargo run -p async-ssh2-lite-demo-smol --bin remote_port_forwarding 127.0.0.1:22 root 8101
*/

use std::env;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::str;

use async_io::Async;
use futures::executor::block_on;
use futures::{AsyncReadExt, AsyncWriteExt};

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
    let remote_port: u16 = env::args()
        .nth(3)
        .unwrap_or_else(|| env::var("REMOTE_PORT").unwrap_or_else(|_| "0".to_owned()))
        .parse()
        .unwrap();

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

    let (mut listener, remote_port) = session
        .channel_forward_listen(remote_port, Some("127.0.0.1"), None)
        .await?;
    println!("run `netstat -tunlp | grep {}` in ssh server", remote_port);
    println!(
        "run `curl http://127.0.0.1:{}/ -v` in ssh server",
        remote_port
    );

    loop {
        match listener.accept().await {
            Ok(mut channel) => {
                let mut buf = vec![0; 64];
                channel.read(&mut buf).await?;
                println!("channel receive {:?}", str::from_utf8(&buf));
                if buf.starts_with(b"GET / HTTP/1.1\r\n") {
                    channel.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;
                } else {
                    channel
                        .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                        .await?;
                    break;
                }
            }
            Err(err) => {
                eprintln!("accept failed, error: {:?}", err);
            }
        }
    }

    println!("done");

    Ok(())
}
