use std::{
    env, error,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

//
const USERNAME: &str = "linuxserver.io";
const PASSWORD: &str = "password";

//
pub(super) fn get_connect_addr() -> Result<SocketAddr, Box<dyn error::Error>> {
    let host = env::var("SSH_SERVER_HOST")
        .ok()
        .as_deref()
        .unwrap_or_else(|| "127.0.0.1")
        .parse::<IpAddr>()?;

    let port = env::var("SSH_SERVER_PORT")?;
    let port = port.parse::<u16>()?;

    Ok(SocketAddr::from((host, port)))
}

pub(super) fn get_username() -> Box<str> {
    env::var("SSH_USERNAME")
        .ok()
        .as_deref()
        .unwrap_or_else(|| USERNAME)
        .into()
}

pub(super) fn get_password() -> Box<str> {
    env::var("SSH_PASSWORD")
        .ok()
        .as_deref()
        .unwrap_or_else(|| PASSWORD)
        .into()
}

pub(super) fn get_privatekey_path() -> PathBuf {
    env::var("SSH_PRIVATEKEY_PATH")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let manifest_path = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
                PathBuf::from(&manifest_dir)
            } else {
                PathBuf::new()
            };

            let keys_dir = manifest_path.join("tests").join("keys");
            let keys_dir = if keys_dir.exists() {
                keys_dir
            } else {
                manifest_path.join("tests").join("keys")
            };

            keys_dir.join("id_rsa")
        })
        .into()
}

pub(super) fn is_internal_openssh_server_docker() -> bool {
    env::var("INTERNAL_OPENSSH_SERVER_DOCKER")
        .ok()
        .map(|x| x == "1")
        == Some(true)
}

//
pub(super) fn init_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}
