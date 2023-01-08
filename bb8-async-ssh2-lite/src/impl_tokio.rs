use std::net::SocketAddr;

use async_ssh2_lite::{AsyncSession, SessionConfiguration, TokioTcpStream};
use async_trait::async_trait;

use crate::{AsyncSessionManagerError, AsyncSessionUserauthType};

//
#[derive(Debug, Clone)]
pub struct AsyncSessionManagerWithTokioTcpStream {
    socket_addr: SocketAddr,
    configuration: Option<SessionConfiguration>,
    username: String,
    userauth_type: AsyncSessionUserauthType,
}

impl AsyncSessionManagerWithTokioTcpStream {
    pub fn new(
        socket_addr: SocketAddr,
        configuration: impl Into<Option<SessionConfiguration>>,
        username: impl AsRef<str>,
        userauth_type: AsyncSessionUserauthType,
    ) -> Self {
        Self {
            socket_addr: socket_addr.into(),
            configuration: configuration.into(),
            username: username.as_ref().into(),
            userauth_type,
        }
    }
}

#[async_trait]
impl bb8::ManageConnection for AsyncSessionManagerWithTokioTcpStream {
    type Connection = AsyncSession<TokioTcpStream>;

    type Error = AsyncSessionManagerError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let mut session = AsyncSession::<TokioTcpStream>::connect(
            self.socket_addr,
            self.configuration.to_owned(),
        )
        .await
        .map_err(AsyncSessionManagerError::ConnectError)?;

        session
            .handshake()
            .await
            .map_err(AsyncSessionManagerError::HandshakeError)?;

        match &self.userauth_type {
            AsyncSessionUserauthType::Password { password } => {
                session
                    .userauth_password(&self.username, password)
                    .await
                    .map_err(AsyncSessionManagerError::UserauthError)?;
            }
            AsyncSessionUserauthType::Agent => {
                session
                    .userauth_agent(&self.username)
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
                        &self.username,
                        pubkey.as_deref(),
                        privatekey,
                        passphrase.as_deref(),
                    )
                    .await
                    .map_err(AsyncSessionManagerError::UserauthError)?;
            }
        }

        if session.authenticated() {
            return Err(AsyncSessionManagerError::AssertAuthenticated);
        }

        Ok(session)
    }

    async fn is_valid(&self, _conn: &mut Self::Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
