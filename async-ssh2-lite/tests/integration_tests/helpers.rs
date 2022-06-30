use std::{
    env, error,
    net::{IpAddr, SocketAddr},
};

//
pub(super) const USERNAME: &str = "linuxserver.io";
pub(super) const PASSWORD: &str = "password";

//
pub(super) fn get_conn_addr() -> Result<SocketAddr, Box<dyn error::Error>> {
    let port = env::var("SSH_SERVER_TCP_PORT")?;

    let ip_addr = "127.0.0.1".parse::<IpAddr>()?;
    let port = port.parse::<u16>()?;

    Ok(SocketAddr::from((ip_addr, port)))
}

pub(super) fn init_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}
