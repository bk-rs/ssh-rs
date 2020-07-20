/*
cargo run -p async-ssh2-lite-demo-smol --bin inspect_ssh_agent
*/

use std::io;

#[cfg(not(unix))]
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::net::UnixListener;
#[cfg(unix)]
use tempfile::tempdir;

use async_io::Async;
use blocking::block_on;

use async_ssh2_lite::AsyncAgent;

fn main() -> io::Result<()> {
    block_on(run())
}

async fn run() -> io::Result<()> {
    let stream = {
        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                let dir = tempdir()?;
                let path = dir.path().join("ssh_agent");
                Async::<UnixListener>::bind(&path)?
            } else {
                Async::<TcpListener>::bind(([127, 0, 0, 1], 0))?
            }
        }
    };

    let mut agent = AsyncAgent::new(stream)?;

    agent.connect().await?;
    agent.list_identities().await?;

    for identity in agent.identities()? {
        println!("identity comment: {}", identity.comment());
    }

    println!("done");

    Ok(())
}
