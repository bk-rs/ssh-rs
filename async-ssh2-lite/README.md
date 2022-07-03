# async-ssh2-lite

* [Cargo package](https://crates.io/crates/async-ssh2-lite)

## Examples

* [Authenticating with password](tests/integration_tests/session__userauth_password.rs)
* [Authenticating with pubkey](tests/integration_tests/session__userauth_pubkey.rs)
* [Authenticating with agent](tests/integration_tests/session__userauth_agent.rs)
* [Inspecting ssh-agent](tests/integration_tests/agent__list_identities.rs)
* [Upload a file](tests/integration_tests/session__scp_send_and_scp_recv.rs)
* [Download a file](tests/integration_tests/session__scp_send_and_scp_recv.rs)
* [Run commands](tests/integration_tests/channel__exec.rs)
* [Inspecting sftp](tests/integration_tests/sftp.rs)
* [Remote port forwarding](tests/integration_tests/session__channel_forward_listen.rs)

* [Through a jump host / bastion host](demos/smol/src/proxy_jump.rs)
