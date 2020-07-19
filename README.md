# async-ssh2-lite

* [Cargo package](https://crates.io/crates/async-ssh2-lite)

## Examples

### smol 

* [Inspecting ssh-agent](demos/smol/src/inspect_ssh_agent.rs)
* [Authenticating with ssh-agent](demos/smol/src/auth_with_ssh_agent.rs)
* [Authenticating with a password](demos/smol/src/auth_with_password.rs)
* [Run commands](demos/smol/src/run_commands.rs)
* [Remote port forwarding](demos/smol/src/remote_port_forwarding.rs)
* [Through a jump host / bastion host](demos/smol/src/proxy_jump.rs)

## Dev

```
cargo test --all-features --all -- --nocapture && \
cargo clippy --all -- -D clippy::all && \
cargo fmt --all -- --check
```

```
cargo build-all-features
cargo test-all-features --all
```
