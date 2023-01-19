use core::{
    cmp::max,
    sync::atomic::{AtomicIsize, Ordering},
};
use std::net::SocketAddr;

use async_ssh2_lite::{AsyncSession, SessionConfiguration, TokioTcpStream};
use async_trait::async_trait;
use tokio_crate as tokio;

use crate::{AsyncSessionManagerError, AsyncSessionUserauthType};

//
#[derive(Debug)]
#[non_exhaustive]
pub struct AsyncSessionManagerWithTokioTcpStream {
    pub socket_addr: SocketAddr,
    pub configuration: Option<SessionConfiguration>,
    pub username: String,
    pub userauth_type: AsyncSessionUserauthType,
    //
    pub max_number_of_unauthenticated_conns: Option<AtomicIsize>,
}

impl Clone for AsyncSessionManagerWithTokioTcpStream {
    fn clone(&self) -> Self {
        Self {
            socket_addr: self.socket_addr.clone(),
            configuration: self.configuration.clone(),
            username: self.username.clone(),
            userauth_type: self.userauth_type.clone(),
            //
            max_number_of_unauthenticated_conns: self
                .max_number_of_unauthenticated_conns
                .as_ref()
                .map(|max_number_of_unauthenticated_conns| {
                    AtomicIsize::new(max_number_of_unauthenticated_conns.load(Ordering::SeqCst))
                }),
        }
    }
}

impl AsyncSessionManagerWithTokioTcpStream {
    pub fn new(
        socket_addr: SocketAddr,
        configuration: impl Into<Option<SessionConfiguration>>,
        username: impl AsRef<str>,
        userauth_type: AsyncSessionUserauthType,
    ) -> Self {
        Self {
            socket_addr,
            configuration: configuration.into(),
            username: username.as_ref().into(),
            userauth_type,
            //
            max_number_of_unauthenticated_conns: None,
        }
    }

    pub fn set_max_number_of_unauthenticated_conns(
        &mut self,
        max_number_of_unauthenticated_conns: usize,
    ) {
        self.max_number_of_unauthenticated_conns = Some(AtomicIsize::new(max(
            1,
            max_number_of_unauthenticated_conns,
        ) as isize));
    }
}

#[async_trait]
impl bb8::ManageConnection for AsyncSessionManagerWithTokioTcpStream {
    type Connection = AsyncSession<TokioTcpStream>;

    type Error = AsyncSessionManagerError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        if let Some(max_number_of_unauthenticated_conns) =
            self.max_number_of_unauthenticated_conns.as_ref()
        {
            loop {
                if max_number_of_unauthenticated_conns.load(Ordering::SeqCst) > 0 {
                    break;
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }

        //
        self.max_number_of_unauthenticated_conns.as_ref().map(
            |max_number_of_unauthenticated_conns| {
                max_number_of_unauthenticated_conns.fetch_sub(1, Ordering::SeqCst)
            },
        );

        //
        match connect_inner(
            self.socket_addr,
            self.configuration.to_owned(),
            &self.username,
            &self.userauth_type,
        )
        .await
        {
            Ok(session) => {
                self.max_number_of_unauthenticated_conns.as_ref().map(
                    |max_number_of_unauthenticated_conns| {
                        max_number_of_unauthenticated_conns.fetch_add(1, Ordering::SeqCst)
                    },
                );

                Ok(session)
            }
            Err(err) => {
                self.max_number_of_unauthenticated_conns.as_ref().map(
                    |max_number_of_unauthenticated_conns| {
                        max_number_of_unauthenticated_conns.fetch_add(1, Ordering::SeqCst)
                    },
                );

                Err(err)
            }
        }
    }

    async fn is_valid(&self, _conn: &mut Self::Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

async fn connect_inner(
    socket_addr: SocketAddr,
    configuration: Option<SessionConfiguration>,
    username: &str,
    userauth_type: &AsyncSessionUserauthType,
) -> Result<AsyncSession<TokioTcpStream>, AsyncSessionManagerError> {
    let mut session = AsyncSession::<TokioTcpStream>::connect(socket_addr, configuration)
        .await
        .map_err(AsyncSessionManagerError::ConnectError)?;

    session
        .handshake()
        .await
        .map_err(AsyncSessionManagerError::HandshakeError)?;

    match userauth_type {
        AsyncSessionUserauthType::Password { password } => {
            session
                .userauth_password(username, password)
                .await
                .map_err(AsyncSessionManagerError::UserauthError)?;
        }
        AsyncSessionUserauthType::Agent => {
            session
                .userauth_agent(username)
                .await
                .map_err(AsyncSessionManagerError::UserauthError)?;
        }
        AsyncSessionUserauthType::PubkeyFile {
            pubkey,
            privatekey,
            passphrase,
        } => {
            session
                .userauth_pubkey_file(
                    username,
                    pubkey.as_deref(),
                    privatekey,
                    passphrase.as_deref(),
                )
                .await
                .map_err(AsyncSessionManagerError::UserauthError)?;
        }
    }

    if !session.authenticated() {
        return Err(AsyncSessionManagerError::AssertAuthenticated);
    }

    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{env, sync::Arc};

    use bb8::ManageConnection as _;
    use futures_util::future::join_all;
    use tokio_crate as tokio;

    #[tokio::test]
    async fn test_max_number_of_unauthenticated_conns() -> Result<(), Box<dyn std::error::Error>> {
        let host = env::var("SSH_SERVER_HOST_AND_PORT").unwrap_or("google.com:443".into());

        let addr = match tokio::net::lookup_host(host).await {
            Ok(mut addrs) => match addrs.next() {
                Some(addr) => addr,
                None => {
                    eprintln!("lookup_host result empty");
                    return Ok(());
                }
            },
            Err(err) => {
                eprintln!("lookup_host failed, err:{err}");
                return Ok(());
            }
        };

        let max_number_of_unauthenticated_conns: usize = 4;

        let mut mgr = AsyncSessionManagerWithTokioTcpStream::new(
            addr,
            None,
            env::var("USER").unwrap_or("root".into()),
            AsyncSessionUserauthType::Agent,
        );
        mgr.set_max_number_of_unauthenticated_conns(max_number_of_unauthenticated_conns);

        let mgr = Arc::new(mgr);

        {
            let mgr = mgr.clone();
            tokio::spawn(async move {
                loop {
                    println!(
                        "max_number_of_unauthenticated_conns:{:?}",
                        mgr.max_number_of_unauthenticated_conns.as_ref()
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            });
        }

        let now = std::time::Instant::now();

        let mut handles = vec![];
        for _ in 0..3 {
            for _ in 0..8 {
                let mgr = mgr.clone();
                let handle = tokio::spawn(async move { mgr.connect().await });
                handles.push(handle);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }
        join_all(handles).await;

        assert_eq!(
            mgr.max_number_of_unauthenticated_conns
                .as_ref()
                .map(|x| x.load(Ordering::SeqCst)),
            Some(max_number_of_unauthenticated_conns as isize)
        );

        let elapsed_dur = now.elapsed();
        println!("elapsed_dur:{elapsed_dur:?}",);
        assert!(elapsed_dur.as_millis() >= (300 * 3 + 100 * 4));

        Ok(())
    }
}
