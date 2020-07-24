/*
cargo run -p async-ssh2-lite-demo-smol --bin sample 127.0.0.1:22 root
*/

use std::env;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::thread;

use async_executor::{Executor, LocalExecutor, Task};
use async_io::Async;
use easy_parallel::Parallel;
use futures::AsyncReadExt;

use async_ssh2_lite::{AsyncSession, SessionConfiguration};

fn main() -> io::Result<()> {
    let ex = Executor::new();
    let local_ex = LocalExecutor::new();
    let (trigger, shutdown) = async_channel::unbounded::<()>();

    let ret_vec: (_, io::Result<()>) = Parallel::new()
        .each(0..4, |_| {
            ex.run(async {
                shutdown
                    .recv()
                    .await
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
            })
        })
        .finish(|| {
            local_ex.run(async {
                run().await?;

                drop(trigger);

                Ok(())
            })
        });

    println!("ret_vec: {:?}", ret_vec);

    Ok(())
}

async fn run() -> io::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| env::var("ADDR").unwrap_or("127.0.0.1:22".to_owned()));
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| env::var("USERNAME").unwrap_or("root".to_owned()));

    let addr = addr.to_socket_addrs().unwrap().next().unwrap();

    //
    let mut receivers = vec![];
    for i in 0..5 {
        let username = username.clone();

        let (sender, receiver) = async_channel::unbounded();
        receivers.push(receiver);

        let task: Task<io::Result<()>> = Task::spawn(async move {
            println!("{} {:?} connect", i, thread::current().id());
            let stream =
                Async::<TcpStream>::connect(addr.to_socket_addrs().unwrap().next().unwrap())
                    .await?;

            let mut session_configuration = SessionConfiguration::new();
            session_configuration.set_timeout(500);
            let mut session = AsyncSession::new(stream, Some(session_configuration))?;

            println!("{} {:?} handshake", i, thread::current().id());
            session.handshake().await?;

            println!("{} {:?} userauth_agent", i, thread::current().id());
            session.userauth_agent(username.as_ref()).await?;

            assert!(session.authenticated());

            println!("{} {:?} channel_session", i, thread::current().id());
            let mut channel = session.channel_session().await?;

            println!("{} {:?} exec", i, thread::current().id());
            channel.exec("hostname").await?;

            println!("{} {:?} read", i, thread::current().id());
            let mut s = String::new();
            channel.read_to_string(&mut s).await?;
            println!("{} hostname: {}", i, s);

            println!("{} {:?} close", i, thread::current().id());
            channel.close().await?;

            Ok(())
        });

        Task::spawn(async move {
            task.await
                .unwrap_or_else(|err| eprintln!("task {} failed, err: {}", i, err));

            sender.send(format!("{} done", i)).await.unwrap()
        })
        .detach();
    }

    for receiver in receivers {
        let msg = receiver.recv().await.unwrap();
        println!("{}", msg);
    }

    Ok(())
}
