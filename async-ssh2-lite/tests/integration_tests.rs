#![cfg(feature = "_integration_tests")]

#[path = "integration_tests"]
mod integration_tests {
    mod helpers;

    #[cfg(test)]
    mod userauth_password;
}
