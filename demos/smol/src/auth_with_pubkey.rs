/*
cargo run -p async-ssh2-lite-demo-smol --bin auth_with_pubkey 127.0.0.1:22 root ~/.ssh/id_rsa
*/

use std::env;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;

use async_io::Async;
use futures::executor::block_on;

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
    let privatekey = env::args()
        .nth(3)
        .unwrap_or_else(|| env::var("PRIVATEKEY").unwrap_or("~/.ssh/id_rsa".to_owned()));
    let passphrase = env::args().nth(3).or_else(|| env::var("PASSPHRASE").ok());

    let addr = addr.to_socket_addrs().unwrap().next().unwrap();

    let stream = Async::<TcpStream>::connect(addr).await?;

    let mut session = AsyncSession::new(stream, None)?;

    session.handshake().await?;

    session
        .userauth_pubkey_file(
            username.as_ref(),
            None,
            Path::new(&privatekey),
            passphrase.as_deref(),
        )
        .await?;

    if !session.authenticated() {
        return Err(session
            .last_error()
            .and_then(|err| Some(io::Error::from(err)))
            .unwrap_or(io::Error::new(
                io::ErrorKind::Other,
                "unknown userauth error",
            )));
    }

    println!("done");

    Ok(())
}
