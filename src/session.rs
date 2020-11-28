use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawSocket, FromRawSocket};

use async_io::Async;
use ssh2::{
    BlockDirections, DisconnectCode, Error, HashType, HostKeyType, KeyboardInteractivePrompt,
    KnownHosts, MethodType, ScpFileStat, Session,
};

use crate::agent::AsyncAgent;
use crate::channel::AsyncChannel;
use crate::listener::AsyncListener;
use crate::sftp::AsyncSftp;

pub struct AsyncSession<S> {
    inner: Session,
    async_io: Arc<Async<S>>,
}

#[cfg(unix)]
impl<S> AsyncSession<S>
where
    S: AsRawFd + FromRawFd + 'static,
{
    pub fn new(stream: Async<S>, configuration: Option<SessionConfiguration>) -> io::Result<Self> {
        let mut session = get_session(configuration)?;
        session.set_tcp_stream(stream.into_inner()?);

        let io = unsafe { S::from_raw_fd(session.as_raw_fd()) };
        let async_io = Arc::new(Async::new(io)?);

        Ok(Self {
            inner: session,
            async_io,
        })
    }
}

#[cfg(windows)]
impl<S> AsyncSession<S>
where
    S: AsRawSocket + FromRawSocket + 'static,
{
    pub fn new(stream: Async<S>, configuration: Option<SessionConfiguration>) -> io::Result<Self> {
        let mut session = get_session(configuration)?;
        session.set_tcp_stream(stream.into_inner()?);

        let io = unsafe { S::from_raw_socket(session.as_raw_socket()) };
        let async_io = Arc::new(Async::new(io)?);

        Ok(Self {
            inner: session,
            async_io,
        })
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

impl<S> AsyncSession<S> {
    pub async fn handshake(&mut self) -> io::Result<()> {
        let inner = &mut self.inner;

        self.async_io
            .write_with(|_| inner.handshake().map_err(|err| err.into()))
            .await
    }

    pub async fn userauth_password(&self, username: &str, password: &str) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .userauth_password(username, password)
                    .map_err(|err| err.into())
            })
            .await
    }

    pub async fn userauth_keyboard_interactive<P: KeyboardInteractivePrompt>(
        &self,
        username: &str,
        prompter: &mut P,
    ) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .userauth_keyboard_interactive(username, prompter)
                    .map_err(|err| err.into())
            })
            .await
    }

    pub async fn userauth_agent(&self, username: &str) -> io::Result<()> {
        let mut agent = self.agent()?;
        agent.connect().await?;
        agent.list_identities().await?;
        let identities = agent.identities()?;
        let identity = match identities.get(0) {
            Some(identity) => identity,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "no identities found in the ssh agent",
                ))
            }
        };
        agent.userauth(username, &identity).await
    }

    pub async fn userauth_pubkey_file(
        &self,
        username: &str,
        pubkey: Option<&Path>,
        privatekey: &Path,
        passphrase: Option<&str>,
    ) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .userauth_pubkey_file(username, pubkey, privatekey, passphrase)
                    .map_err(|err| err.into())
            })
            .await
    }

    #[cfg(unix)]
    pub async fn userauth_pubkey_memory(
        &self,
        username: &str,
        pubkeydata: Option<&str>,
        privatekeydata: &str,
        passphrase: Option<&str>,
    ) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .userauth_pubkey_memory(username, pubkeydata, privatekeydata, passphrase)
                    .map_err(|err| err.into())
            })
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
    ) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .userauth_hostbased_file(
                        username,
                        publickey,
                        privatekey,
                        passphrase,
                        hostname,
                        local_username,
                    )
                    .map_err(|err| err.into())
            })
            .await
    }

    pub fn authenticated(&self) -> bool {
        self.inner.authenticated()
    }

    pub async fn auth_methods(&self, username: &str) -> io::Result<&str> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.auth_methods(username).map_err(|err| err.into()))
            .await
    }

    pub async fn method_pref(&self, method_type: MethodType, prefs: &str) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .method_pref(method_type, prefs)
                    .map_err(|err| err.into())
            })
            .await
    }

    pub fn methods(&self, method_type: MethodType) -> Option<&str> {
        self.inner.methods(method_type)
    }

    pub async fn supported_algs(&self, method_type: MethodType) -> io::Result<Vec<&'static str>> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.supported_algs(method_type).map_err(|err| err.into()))
            .await
    }

    pub fn agent(&self) -> io::Result<AsyncAgent<S>> {
        let ret = self.inner.agent().map_err(|err| err.into());

        ret.and_then(|agent| Ok(AsyncAgent::from_parts(agent, self.async_io.clone())))
    }

    pub fn known_hosts(&self) -> io::Result<KnownHosts> {
        self.inner.known_hosts().map_err(|err| err.into())
    }

    pub async fn channel_session(&self) -> io::Result<AsyncChannel<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| inner.channel_session().map_err(|err| err.into()))
            .await;

        ret.and_then(|channel| Ok(AsyncChannel::from_parts(channel, self.async_io.clone())))
    }

    pub async fn channel_direct_tcpip(
        &self,
        host: &str,
        port: u16,
        src: Option<(&str, u16)>,
    ) -> io::Result<AsyncChannel<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| {
                inner
                    .channel_direct_tcpip(host, port, src)
                    .map_err(|err| err.into())
            })
            .await;

        ret.and_then(|channel| Ok(AsyncChannel::from_parts(channel, self.async_io.clone())))
    }

    pub async fn channel_forward_listen(
        &self,
        remote_port: u16,
        host: Option<&str>,
        queue_maxsize: Option<u32>,
    ) -> io::Result<(AsyncListener<S>, u16)> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| {
                inner
                    .channel_forward_listen(remote_port, host, queue_maxsize)
                    .map_err(|err| err.into())
            })
            .await;

        ret.and_then(|(listener, port)| {
            Ok((
                AsyncListener::from_parts(listener, self.async_io.clone()),
                port,
            ))
        })
    }

    pub async fn scp_recv(&self, path: &Path) -> io::Result<(AsyncChannel<S>, ScpFileStat)> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| inner.scp_recv(path).map_err(|err| err.into()))
            .await;

        ret.and_then(|(channel, scp_file_stat)| {
            Ok((
                AsyncChannel::from_parts(channel, self.async_io.clone()),
                scp_file_stat,
            ))
        })
    }

    pub async fn scp_send(
        &self,
        remote_path: &Path,
        mode: i32,
        size: u64,
        times: Option<(u64, u64)>,
    ) -> io::Result<AsyncChannel<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| {
                inner
                    .scp_send(remote_path, mode, size, times)
                    .map_err(|err| err.into())
            })
            .await;

        ret.and_then(|channel| Ok(AsyncChannel::from_parts(channel, self.async_io.clone())))
    }

    pub async fn sftp(&self) -> io::Result<AsyncSftp<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| inner.sftp().map_err(|err| err.into()))
            .await;

        ret.and_then(|sftp| Ok(AsyncSftp::from_parts(sftp, self.async_io.clone())))
    }

    pub async fn channel_open(
        &self,
        channel_type: &str,
        window_size: u32,
        packet_size: u32,
        message: Option<&str>,
    ) -> io::Result<AsyncChannel<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| {
                inner
                    .channel_open(channel_type, window_size, packet_size, message)
                    .map_err(|err| err.into())
            })
            .await;

        ret.and_then(|channel| Ok(AsyncChannel::from_parts(channel, self.async_io.clone())))
    }

    pub fn host_key(&self) -> Option<(&[u8], HostKeyType)> {
        self.inner.host_key()
    }

    pub fn host_key_hash(&self, hash: HashType) -> Option<&[u8]> {
        self.inner.host_key_hash(hash)
    }

    pub async fn keepalive_send(&self) -> io::Result<u32> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.keepalive_send().map_err(|err| err.into()))
            .await
    }

    pub async fn disconnect(
        &self,
        reason: Option<DisconnectCode>,
        description: &str,
        lang: Option<&str>,
    ) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .disconnect(reason, description, lang)
                    .map_err(|err| err.into())
            })
            .await
    }

    pub fn block_directions(&self) -> BlockDirections {
        self.inner.block_directions()
    }
}

//
// extension
//
impl<S> AsyncSession<S> {
    pub fn last_error(&self) -> Option<Error> {
        Error::last_session_error(&self.inner)
    }

    pub async fn userauth_agent_with_try_next(&self, username: &str) -> io::Result<()> {
        let mut agent = self.agent()?;
        agent.connect().await?;
        agent.list_identities().await?;
        let identities = agent.identities()?;

        if identities.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "no identities found in the ssh agent",
            ));
        }

        for identity in identities {
            match agent.userauth(username, &identity).await {
                Ok(_) => {
                    if self.authenticated() {
                        return Ok(());
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }

        return Err(io::Error::new(
            io::ErrorKind::Other,
            "all identities cannot authenticated",
        ));
    }
}

//
//
//
#[derive(Default, Clone)]
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

#[derive(Clone)]
struct SessionKeepaliveConfiguration {
    want_reply: bool,
    interval: u32,
}

pub(crate) fn get_session(configuration: Option<SessionConfiguration>) -> io::Result<Session> {
    let session = Session::new()?;
    session.set_blocking(false);

    if let Some(configuration) = configuration {
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
