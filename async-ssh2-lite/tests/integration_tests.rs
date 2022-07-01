#![cfg(feature = "_integration_tests")]

#[path = "integration_tests"]
mod integration_tests {
    mod helpers;

    #[cfg(test)]
    #[allow(non_snake_case)]
    mod agent__list_identities;

    #[cfg(test)]
    #[allow(non_snake_case)]
    mod channel__exec;

    #[cfg(test)]
    #[allow(non_snake_case)]
    mod session__scp_recv;

    #[cfg(test)]
    #[allow(non_snake_case)]
    mod session__scp_send;

    #[cfg(test)]
    #[allow(non_snake_case)]
    mod session__userauth_password;

    #[cfg(test)]
    #[allow(non_snake_case)]
    mod session__userauth_pubkey;

    #[cfg(test)]
    #[allow(non_snake_case)]
    mod session__userauth_agent;

    #[cfg(test)]
    mod sftp;
}
