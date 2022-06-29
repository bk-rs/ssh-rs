/*
cargo run -p async-ssh2-lite-demo-smol --bin auth_with_password 127.0.0.1:22 root 123456
*/

use std::env;
use std::error;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};

use async_io::Async;
use futures::executor::block_on;

use async_ssh2_lite::AsyncSession;

fn main() -> Result<(), Box<dyn error::Error>> {
    block_on(run())
}

async fn run() -> Result<(), Box<dyn error::Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:22".to_owned()));
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| env::var("USERNAME").unwrap_or_else(|_| "root".to_owned()));
    let password = env::args()
        .nth(3)
        .unwrap_or_else(|| env::var("PASSWORD").unwrap_or_else(|_| "123456".to_owned()));

    let addr = addr.to_socket_addrs().unwrap().next().unwrap();

    let stream = Async::<TcpStream>::connect(addr).await?;

    let mut session = AsyncSession::new(stream, None)?;

    session.handshake().await?;

    session
        .userauth_password(username.as_ref(), password.as_ref())
        .await?;

    if !session.authenticated() {
        return Err(session
            .last_error()
            .map(io::Error::from)
            .unwrap_or_else(|| io::Error::new(io::ErrorKind::Other, "unknown userauth error"))
            .into());
    }

    println!("done");

    Ok(())
}
