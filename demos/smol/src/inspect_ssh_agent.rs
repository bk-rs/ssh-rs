/*
cargo run -p async-ssh2-lite-demo-smol --bin inspect_ssh_agent 127.0.0.1:22
*/

use std::env;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};

use async_io::Async;
use blocking::block_on;

use async_ssh2_lite::AsyncSession;

fn main() -> io::Result<()> {
    block_on(run())
}

async fn run() -> io::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| env::var("ADDR").unwrap_or("127.0.0.1:22".to_owned()));

    let addr = addr.to_socket_addrs().unwrap().next().unwrap();

    let stream = Async::<TcpStream>::connect(addr).await?;

    let session = AsyncSession::new(stream, None)?;

    let mut agent = session.agent()?;

    agent.connect().await?;
    agent.list_identities().await?;

    for identity in agent.identities()? {
        println!("identity comment: {}", identity.comment());
    }

    println!("done");

    Ok(())
}
