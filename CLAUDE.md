# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust workspace containing two main crates for asynchronous SSH2 operations:
- `async-ssh2-lite`: Core async SSH2 client library with support for both async-io and tokio runtimes
- `bb8-async-ssh2-lite`: Connection pooling for async-ssh2-lite using bb8

## Development Commands

### Linting and Formatting
```bash
cargo clippy --all-features --tests -- -D clippy::all
cargo +nightly clippy --all-features --tests -- -D clippy::all
cargo fmt -- --check
```

### Building and Testing
```bash
cargo build-all-features
cargo test-all-features -- --nocapture
```

### Integration Tests
Run the automated integration test script:
```bash
./async-ssh2-lite/tests/run_integration_tests.sh
```

For manual integration testing with a specific SSH server:
```bash
SSH_SERVER_HOST=127.0.0.1 SSH_SERVER_PORT=22 SSH_USERNAME=xx SSH_PASSWORD=xxx SSH_PRIVATEKEY_PATH=~/.ssh/id_rsa cargo test -p async-ssh2-lite --features _integration_tests,async-io,tokio -- --nocapture
```

Run a specific integration test:
```bash
SSH_SERVER_HOST=127.0.0.1 SSH_SERVER_PORT=22 SSH_USERNAME=xx SSH_PRIVATEKEY_PATH=~/.ssh/id_rsa cargo test -p async-ssh2-lite --features _integration_tests,async-io,tokio -- integration_tests::session__scp_send_and_scp_recv --nocapture
```

## Architecture

### async-ssh2-lite
The core library provides async wrappers around ssh2 functionality:
- **Session management** (`session.rs`): Handles SSH connections and authentication
- **Channel operations** (`channel.rs`): Execute commands and manage data streams
- **SFTP support** (`sftp.rs`): File transfer operations
- **Agent support** (`agent.rs`): SSH agent authentication
- **Port forwarding** (`listener.rs`): Remote and local port forwarding

The library supports two async runtimes through feature flags:
- `async-io`: For async-std/smol runtime compatibility
- `tokio`: For tokio runtime compatibility

Session streams are implemented in `session_stream/` with runtime-specific implementations.

### bb8-async-ssh2-lite
Provides connection pooling for SSH sessions using the bb8 pool manager. Includes managers for both sessions and SFTP connections.

### Test Infrastructure
- Integration tests are in `async-ssh2-lite/tests/integration_tests/`
- Test SSH keys are provided in `keys/` directory
- Docker-based OpenSSH server configurations in `openssh_server_docker/` for testing

## Key Features and Capabilities
- Password, public key, and agent-based authentication
- Command execution via SSH channels
- SCP file upload/download
- SFTP operations
- SSH agent integration
- Remote port forwarding
- Proxy jump host support

## Important Notes
- The workspace uses Rust 2021 edition
- Integration tests require special feature flags (`_integration_tests`)
- The library wraps the synchronous ssh2 crate with async runtime support
- Both async-io and tokio runtime support can be enabled simultaneously