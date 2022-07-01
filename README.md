## Dev

```
cargo clippy --all-features --tests -- -D clippy::all
cargo +nightly clippy --all-features --tests -- -D clippy::all

cargo fmt -- --check

cargo build-all-features
cargo test-all-features -- --nocapture
```

```
SSH_SERVER_HOST=127.0.0.1 SSH_SERVER_PORT=22 SSH_USERNAME=xx SSH_PASSWORD=xxx SSH_PRIVATEKEY_PATH=~/.ssh/id_rsa cargo test -p async-ssh2-lite --features _integration_tests,async-io,tokio -- --nocapture

SSH_SERVER_HOST=127.0.0.1 SSH_SERVER_PORT=22 SSH_USERNAME=xx SSH_PASSWORD=xxx SSH_PRIVATEKEY_PATH=~/.ssh/id_rsa cargo test -p async-ssh2-lite --features _integration_tests,async-io,tokio -- integration_tests::agent__list_identities --nocapture
```
