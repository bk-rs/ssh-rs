use std::{
    env, error,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

//
pub(super) fn get_connect_addr() -> Result<SocketAddr, Box<dyn error::Error>> {
    let host = env::var("SSH_SERVER_HOST")
        .expect("Missing SSH_SERVER_HOST")
        .parse::<IpAddr>()?;

    let port = env::var("SSH_SERVER_PORT").expect("Missing SSH_SERVER_PORT");
    let port = port.parse::<u16>()?;

    Ok(SocketAddr::from((host, port)))
}

pub(super) fn get_username() -> Box<str> {
    env::var("SSH_USERNAME")
        .expect("Missing SSH_USERNAME")
        .into()
}

pub(super) fn get_password() -> Option<Box<str>> {
    env::var("SSH_PASSWORD").ok().map(Into::into)
}

pub(super) fn get_privatekey_path() -> PathBuf {
    if is_internal_test_openssh_server() {
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
    } else {
        env::var("SSH_PRIVATEKEY_PATH")
            .ok()
            .map(PathBuf::from)
            .expect("Missing SSH_PRIVATEKEY_PATH")
    }
}

pub(super) fn is_internal_test_openssh_server() -> bool {
    env::var("IS_INTERNAL_TEST_OPENSSH_SERVER")
        .ok()
        .map(|x| x == "1")
        == Some(true)
}

//
pub(super) fn init_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}

//
pub(super) fn get_listen_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], portpicker::pick_unused_port().unwrap()))
}
