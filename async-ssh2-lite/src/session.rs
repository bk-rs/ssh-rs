use core::time::Duration;
use std::{path::Path, sync::Arc};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;
#[cfg(windows)]
use std::os::windows::io::{AsRawSocket, BorrowedSocket};

use ssh2::{
    BlockDirections, DisconnectCode, Error as Ssh2Error, HashType, HostKeyType,
    KeyboardInteractivePrompt, KnownHosts, MethodType, PublicKey, ScpFileStat, Session,
};

use crate::{
    agent::AsyncAgent, channel::AsyncChannel, error::Error, listener::AsyncListener,
    session_stream::AsyncSessionStream, sftp::AsyncSftp,
};

//
pub struct AsyncSession<S> {
    inner: Session,
    stream: Arc<S>,
}

impl<S> Clone for AsyncSession<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            stream: self.stream.clone(),
        }
    }
}

#[cfg(unix)]
impl<S> AsyncSession<S>
where
    S: AsRawFd + 'static,
{
    pub fn new(
        stream: S,
        configuration: impl Into<Option<SessionConfiguration>>,
    ) -> Result<Self, Error> {
        let mut session = get_session(configuration)?;
        session.set_tcp_stream(stream.as_raw_fd());

        let stream = Arc::new(stream);

        Ok(Self {
            inner: session,
            stream,
        })
    }
}

#[cfg(windows)]
impl<S> AsyncSession<S>
where
    S: AsRawSocket + 'static,
{
    pub fn new(
        stream: S,
        configuration: impl Into<Option<SessionConfiguration>>,
    ) -> Result<Self, Error> {
        let mut session = get_session(configuration)?;
        session.set_tcp_stream(unsafe { BorrowedSocket::borrow_raw(stream.as_raw_socket()) });

        let stream = Arc::new(stream);

        Ok(Self {
            inner: session,
            stream,
        })
    }
}

#[cfg(feature = "async-io")]
impl AsyncSession<crate::AsyncIoTcpStream> {
    pub async fn connect<A: Into<std::net::SocketAddr>>(
        addr: A,
        configuration: impl Into<Option<SessionConfiguration>>,
    ) -> Result<Self, Error> {
        let stream = crate::AsyncIoTcpStream::connect(addr).await?;

        Self::new(stream, configuration)
    }
}

#[cfg(all(unix, feature = "async-io"))]
impl AsyncSession<crate::AsyncIoUnixStream> {
    #[cfg(unix)]
    pub async fn connect<P: AsRef<Path>>(
        path: P,
        configuration: impl Into<Option<SessionConfiguration>>,
    ) -> Result<Self, Error> {
        let stream = crate::AsyncIoUnixStream::connect(path).await?;

        Self::new(stream, configuration)
    }
}

#[cfg(feature = "tokio")]
impl AsyncSession<crate::TokioTcpStream> {
    pub async fn connect<A: Into<std::net::SocketAddr>>(
        addr: A,
        configuration: impl Into<Option<SessionConfiguration>>,
    ) -> Result<Self, Error> {
        let stream = crate::TokioTcpStream::connect(addr.into()).await?;

        Self::new(stream, configuration)
    }
}

#[cfg(all(unix, feature = "tokio"))]
impl AsyncSession<crate::TokioUnixStream> {
    #[cfg(unix)]
    pub async fn connect<P: AsRef<Path>>(
        path: P,
        configuration: impl Into<Option<SessionConfiguration>>,
    ) -> Result<Self, Error> {
        let stream = crate::TokioUnixStream::connect(path).await?;

        Self::new(stream, configuration)
    }
}

impl<S> AsyncSession<S> {
    pub fn is_blocking(&self) -> bool {
        self.inner.is_blocking()
    }

    pub fn banner(&self) -> Option<&str> {
        self.inner.banner()
    }

    pub fn banner_bytes(&self) -> Option<&[u8]> {
        self.inner.banner_bytes()
    }

    pub fn timeout(&self) -> u32 {
        self.inner.timeout()
    }
}

impl<S> AsyncSession<S>
where
    S: AsyncSessionStream + Send + Sync + 'static,
{
    pub async fn handshake(&mut self) -> Result<(), Error> {
        let sess = self.inner.clone();
        self.stream.rw_with(|| self.inner.handshake(), &sess).await
    }

    pub async fn userauth_password(&self, username: &str, password: &str) -> Result<(), Error> {
        self.stream
            .rw_with(
                || self.inner.userauth_password(username, password),
                &self.inner,
            )
            .await
    }

    #[allow(unknown_lints)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub async fn userauth_keyboard_interactive<P: KeyboardInteractivePrompt + Send>(
        &self,
        username: &str,
        prompter: &mut P,
    ) -> Result<(), Error> {
        self.stream
            .rw_with(
                || self.inner.userauth_keyboard_interactive(username, prompter),
                &self.inner,
            )
            .await
    }

    pub async fn userauth_agent(&self, username: &str) -> Result<(), Error> {
        let mut agent = self.agent()?;
        agent.connect().await?;
        agent.list_identities().await?;
        let identities = agent.identities()?;
        let identity = match identities.get(0) {
            Some(identity) => identity,
            None => return Err(Error::Other("no identities found in the ssh agent".into())),
        };
        agent.userauth(username, identity).await
    }

    pub async fn userauth_pubkey_file(
        &self,
        username: &str,
        pubkey: Option<&Path>,
        privatekey: &Path,
        passphrase: Option<&str>,
    ) -> Result<(), Error> {
        self.stream
            .rw_with(
                || {
                    self.inner
                        .userauth_pubkey_file(username, pubkey, privatekey, passphrase)
                },
                &self.inner,
            )
            .await
    }

    #[cfg(any(unix, feature = "vendored-openssl", feature = "openssl-on-win32"))]
    pub async fn userauth_pubkey_memory(
        &self,
        username: &str,
        pubkeydata: Option<&str>,
        privatekeydata: &str,
        passphrase: Option<&str>,
    ) -> Result<(), Error> {
        self.stream
            .rw_with(
                || {
                    self.inner.userauth_pubkey_memory(
                        username,
                        pubkeydata,
                        privatekeydata,
                        passphrase,
                    )
                },
                &self.inner,
            )
            .await
    }

    pub async fn userauth_hostbased_file(
        &self,
        username: &str,
        publickey: &Path,
        privatekey: &Path,
        passphrase: Option<&str>,
        hostname: &str,
        local_username: Option<&str>,
    ) -> Result<(), Error> {
        self.stream
            .rw_with(
                || {
                    self.inner.userauth_hostbased_file(
                        username,
                        publickey,
                        privatekey,
                        passphrase,
                        hostname,
                        local_username,
                    )
                },
                &self.inner,
            )
            .await
    }

    pub fn authenticated(&self) -> bool {
        self.inner.authenticated()
    }

    pub async fn auth_methods<'a>(&'a self, username: &'a str) -> Result<&str, Error> {
        self.stream
            .rw_with(|| self.inner.auth_methods(username), &self.inner)
            .await
    }

    pub async fn method_pref(&self, method_type: MethodType, prefs: &str) -> Result<(), Error> {
        self.stream
            .rw_with(|| self.inner.method_pref(method_type, prefs), &self.inner)
            .await
    }

    pub fn methods(&self, method_type: MethodType) -> Option<&str> {
        self.inner.methods(method_type)
    }

    pub async fn supported_algs(
        &self,
        method_type: MethodType,
    ) -> Result<Vec<&'static str>, Error> {
        self.stream
            .rw_with(|| self.inner.supported_algs(method_type), &self.inner)
            .await
    }

    pub fn agent(&self) -> Result<AsyncAgent<S>, Error> {
        let agent = self.inner.agent()?;

        Ok(AsyncAgent::from_parts(
            agent,
            self.inner.clone(),
            self.stream.clone(),
        ))
    }

    pub fn known_hosts(&self) -> Result<KnownHosts, Error> {
        self.inner.known_hosts().map_err(Into::into)
    }

    pub async fn channel_session(&self) -> Result<AsyncChannel<S>, Error> {
        let channel = self
            .stream
            .rw_with(|| self.inner.channel_session(), &self.inner)
            .await?;

        Ok(AsyncChannel::from_parts(
            channel,
            self.inner.clone(),
            self.stream.clone(),
        ))
    }

    pub async fn channel_direct_tcpip(
        &self,
        host: &str,
        port: u16,
        src: Option<(&str, u16)>,
    ) -> Result<AsyncChannel<S>, Error> {
        let channel = self
            .stream
            .rw_with(
                || self.inner.channel_direct_tcpip(host, port, src),
                &self.inner,
            )
            .await?;

        Ok(AsyncChannel::from_parts(
            channel,
            self.inner.clone(),
            self.stream.clone(),
        ))
    }

    pub async fn channel_forward_listen(
        &self,
        remote_port: u16,
        host: Option<&str>,
        queue_maxsize: Option<u32>,
    ) -> Result<(AsyncListener<S>, u16), Error> {
        let (listener, port) = self
            .stream
            .rw_with(
                || {
                    self.inner
                        .channel_forward_listen(remote_port, host, queue_maxsize)
                },
                &self.inner,
            )
            .await?;

        Ok((
            AsyncListener::from_parts(listener, self.inner.clone(), self.stream.clone()),
            port,
        ))
    }

    pub async fn scp_recv(&self, path: &Path) -> Result<(AsyncChannel<S>, ScpFileStat), Error> {
        let (channel, scp_file_stat) = self
            .stream
            .rw_with(|| self.inner.scp_recv(path), &self.inner)
            .await?;

        Ok((
            AsyncChannel::from_parts(channel, self.inner.clone(), self.stream.clone()),
            scp_file_stat,
        ))
    }

    pub async fn scp_send(
        &self,
        remote_path: &Path,
        mode: i32,
        size: u64,
        times: Option<(u64, u64)>,
    ) -> Result<AsyncChannel<S>, Error> {
        let channel = self
            .stream
            .rw_with(
                || self.inner.scp_send(remote_path, mode, size, times),
                &self.inner,
            )
            .await?;

        Ok(AsyncChannel::from_parts(
            channel,
            self.inner.clone(),
            self.stream.clone(),
        ))
    }

    pub async fn sftp(&self) -> Result<AsyncSftp<S>, Error> {
        let sftp = self
            .stream
            .rw_with(|| self.inner.sftp(), &self.inner)
            .await?;

        Ok(AsyncSftp::from_parts(
            sftp,
            self.inner.clone(),
            self.stream.clone(),
        ))
    }

    pub async fn channel_open(
        &self,
        channel_type: &str,
        window_size: u32,
        packet_size: u32,
        message: Option<&str>,
    ) -> Result<AsyncChannel<S>, Error> {
        let channel = self
            .stream
            .rw_with(
                || {
                    self.inner
                        .channel_open(channel_type, window_size, packet_size, message)
                },
                &self.inner,
            )
            .await?;

        Ok(AsyncChannel::from_parts(
            channel,
            self.inner.clone(),
            self.stream.clone(),
        ))
    }

    pub fn host_key(&self) -> Option<(&[u8], HostKeyType)> {
        self.inner.host_key()
    }

    pub fn host_key_hash(&self, hash: HashType) -> Option<&[u8]> {
        self.inner.host_key_hash(hash)
    }

    pub async fn keepalive_send(&self) -> Result<u32, Error> {
        self.stream
            .rw_with(|| self.inner.keepalive_send(), &self.inner)
            .await
    }

    pub async fn disconnect(
        &self,
        reason: Option<DisconnectCode>,
        description: &str,
        lang: Option<&str>,
    ) -> Result<(), Error> {
        self.stream
            .rw_with(
                || self.inner.disconnect(reason, description, lang),
                &self.inner,
            )
            .await
    }

    pub fn block_directions(&self) -> BlockDirections {
        self.inner.block_directions()
    }
}

#[cfg(feature = "tokio")]
impl<S> AsyncSession<S>
where
    S: AsyncSessionStream + Send + Sync + 'static,
{
    pub async fn remote_port_forwarding(
        &self,
        remote_port: u16,
        host: Option<&str>,
        queue_maxsize: Option<u32>,
        local: crate::util::ConnectInfo,
    ) -> Result<(), Error> {
        use std::io::Error as IoError;

        use futures_util::{select, FutureExt as _};
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

        #[cfg(unix)]
        use crate::TokioUnixStream;
        use crate::{util::ConnectInfo, TokioTcpStream};

        match local {
            ConnectInfo::Tcp(addr) => {
                let (mut listener, _remote_port) = self
                    .channel_forward_listen(remote_port, host, queue_maxsize)
                    .await?;

                // TODO, tokio::io::copy_bidirectional not working

                loop {
                    match listener.accept().await {
                        Ok(mut channel) => {
                            let join_handle: tokio::task::JoinHandle<Result<(), IoError>> =
                                tokio::task::spawn(async move {
                                    let mut stream = TokioTcpStream::connect(addr).await?;

                                    let mut buf_channel = vec![0; 2048];
                                    let mut buf_stream = vec![0; 2048];

                                    loop {
                                        select! {
                                            ret_channel_read = futures_util::AsyncReadExt::read(&mut channel, &mut buf_channel).fuse() => match ret_channel_read {
                                                Ok(0)  => {
                                                    break
                                                },
                                                Ok(n) => {
                                                    #[allow(clippy::map_identity)]
                                                    stream.write(&buf_channel[..n]).await.map(|_| ()).map_err(|err| {
                                                        // TODO, log
                                                        err
                                                    })?
                                                },
                                                Err(err) =>  {
                                                    return Err(err);
                                                }
                                            },
                                            ret_stream_read = stream.read(&mut buf_stream).fuse() => match ret_stream_read {
                                                Ok(0)  => {
                                                    break
                                                },
                                                Ok(n) => {
                                                    #[allow(clippy::map_identity)]
                                                    futures_util::AsyncWriteExt::write(&mut channel,&buf_stream[..n]).await.map(|_| ()).map_err(|err| {
                                                        // TODO, log
                                                        err
                                                    })?
                                                },
                                                Err(err) => {
                                                    return Err(err);
                                                }
                                            },
                                        }
                                    }

                                    Result::<_, IoError>::Ok(())
                                });
                            match join_handle.await {
                                Ok(_) => {}
                                Err(err) => {
                                    eprintln!("join_handle failed, err:{err:?}");
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("listener.accept failed, err:{err:?}");
                        }
                    }
                }
            }
            #[cfg(unix)]
            ConnectInfo::Unix(path) => {
                let (mut listener, _remote_port) = self
                    .channel_forward_listen(remote_port, host, queue_maxsize)
                    .await?;

                // TODO, tokio::io::copy_bidirectional not working

                loop {
                    match listener.accept().await {
                        Ok(mut channel) => {
                            let path = path.clone();
                            let join_handle: tokio::task::JoinHandle<Result<(), IoError>> =
                                tokio::task::spawn(async move {
                                    let mut stream = TokioUnixStream::connect(path).await?;

                                    let mut buf_channel = vec![0; 2048];
                                    let mut buf_stream = vec![0; 2048];

                                    loop {
                                        select! {
                                            ret_channel_read = futures_util::AsyncReadExt::read(&mut channel, &mut buf_channel).fuse() => match ret_channel_read {
                                                Ok(0)  => {
                                                    break
                                                },
                                                Ok(n) => {
                                                    #[allow(clippy::map_identity)]
                                                    stream.write(&buf_channel[..n]).await.map(|_| ()).map_err(|err| {
                                                        // TODO, log
                                                        err
                                                    })?
                                                },
                                                Err(err) =>  {
                                                    return Err(err);
                                                }
                                            },
                                            ret_stream_read = stream.read(&mut buf_stream).fuse() => match ret_stream_read {
                                                Ok(0)  => {
                                                    break
                                                },
                                                Ok(n) => {
                                                    #[allow(clippy::map_identity)]
                                                    futures_util::AsyncWriteExt::write(&mut channel,&buf_stream[..n]).await.map(|_| ()).map_err(|err| {
                                                        // TODO, log
                                                        err
                                                    })?
                                                },
                                                Err(err) => {
                                                    return Err(err);
                                                }
                                            },
                                        }
                                    }

                                    Result::<_, IoError>::Ok(())
                                });
                            match join_handle.await {
                                Ok(_) => {}
                                Err(err) => {
                                    eprintln!("join_handle failed, err:{err:?}");
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("listener.accept failed, err:{err:?}");
                        }
                    }
                }
            }
        }
    }
}

//
// extension
//
impl<S> AsyncSession<S> {
    pub fn last_error(&self) -> Option<Ssh2Error> {
        Ssh2Error::last_session_error(&self.inner)
    }
}

impl<S> AsyncSession<S>
where
    S: AsyncSessionStream + Send + Sync + 'static,
{
    pub async fn userauth_agent_with_try_next(&self, username: &str) -> Result<(), Error> {
        self.userauth_agent_with_try_next_with_callback(username, |identities| identities)
            .await
    }

    pub async fn userauth_agent_with_try_next_with_callback<CB>(
        &self,
        username: &str,
        mut cb: CB,
    ) -> Result<(), Error>
    where
        CB: FnMut(Vec<PublicKey>) -> Vec<PublicKey>,
    {
        let mut agent = self.agent()?;
        agent.connect().await?;
        agent.list_identities().await?;
        let identities = agent.identities()?;

        if identities.is_empty() {
            return Err(Error::Other("no identities found in the ssh agent".into()));
        }

        let identities = cb(identities);

        for identity in identities {
            match agent.userauth(username, &identity).await {
                Ok(_) => {
                    if self.authenticated() {
                        return Ok(());
                    }
                }
                Err(_err) => {
                    continue;
                }
            }
        }

        Err(Error::Other("all identities cannot authenticated".into()))
    }
}

//
//
//
#[derive(Debug, Clone, Default)]
pub struct SessionConfiguration {
    banner: Option<String>,
    allow_sigpipe: Option<bool>,
    compress: Option<bool>,
    timeout: Option<Duration>,
    keepalive: Option<SessionKeepaliveConfiguration>,
}
impl SessionConfiguration {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_banner(&mut self, banner: &str) {
        self.banner = Some(banner.to_owned());
    }

    pub fn set_allow_sigpipe(&mut self, block: bool) {
        self.allow_sigpipe = Some(block);
    }

    pub fn set_compress(&mut self, compress: bool) {
        self.compress = Some(compress);
    }

    pub fn set_timeout(&mut self, timeout_ms: u32) {
        self.timeout = Some(Duration::from_millis(timeout_ms as u64));
    }

    pub fn set_keepalive(&mut self, want_reply: bool, interval: u32) {
        self.keepalive = Some(SessionKeepaliveConfiguration {
            want_reply,
            interval,
        });
    }
}

#[derive(Debug, Clone)]
struct SessionKeepaliveConfiguration {
    want_reply: bool,
    interval: u32,
}

pub(crate) fn get_session(
    configuration: impl Into<Option<SessionConfiguration>>,
) -> Result<Session, Error> {
    let session = Session::new()?;
    session.set_blocking(false);

    if let Some(configuration) = configuration.into() {
        if let Some(banner) = configuration.banner {
            session.set_banner(banner.as_ref())?;
        }
        if let Some(allow_sigpipe) = configuration.allow_sigpipe {
            session.set_allow_sigpipe(allow_sigpipe);
        }
        if let Some(compress) = configuration.compress {
            session.set_compress(compress);
        }
        if let Some(timeout) = configuration.timeout {
            session.set_timeout(timeout.as_millis() as u32);
        }
        if let Some(keepalive) = configuration.keepalive {
            session.set_keepalive(keepalive.want_reply, keepalive.interval);
        }
    }

    Ok(session)
}
