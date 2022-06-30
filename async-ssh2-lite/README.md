# async-ssh2-lite

* [Cargo package](https://crates.io/crates/async-ssh2-lite)

## Examples

* [Authenticating with password](tests/integration_tests/userauth_password.rs)
* [Authenticating with pubkey](tests/integration_tests/userauth_pubkey.rs)
* [Authenticating with agent](tests/integration_tests/userauth_agent.rs)

* [Inspecting ssh-agent](demos/smol/src/inspect_ssh_agent.rs)
* [Run commands](demos/smol/src/run_commands.rs)
* [Remote port forwarding](demos/smol/src/remote_port_forwarding.rs)
* [Through a jump host / bastion host](demos/smol/src/proxy_jump.rs)
* [Inspecting sftp](demos/smol/src/inspect_sftp.rs)
* [Upload a file](demos/smol/src/upload_file.rs)
* [Download a file](demos/smol/src/download_file.rs)
