#![allow(non_snake_case)]
#![cfg(feature = "_integration_tests")]

#[path = "integration_tests"]
mod integration_tests {
    mod helpers;

    #[cfg(test)]
    mod agent__list_identities;

    #[cfg(test)]
    mod channel__exec;

    #[cfg(test)]
    mod remote_port_forwarding;

    #[cfg(test)]
    mod session__channel_forward_listen;

    #[cfg(test)]
    mod session__scp_send_and_scp_recv;

    #[cfg(test)]
    mod session__userauth_password;

    #[cfg(test)]
    mod session__userauth_pubkey;

    #[cfg(test)]
    mod session__userauth_agent;

    #[cfg(test)]
    mod sftp;
}
