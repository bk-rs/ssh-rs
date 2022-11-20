#!/usr/bin/env bash

set -ex

version="${1:-8.8_p1-r1-ls82}"

# RUST_BACKTRACE=full ./tests/run_integration_tests.sh
# RUST_LOG=trace ./tests/run_integration_tests.sh

script_path=$(cd $(dirname $0) ; pwd -P)
script_path_root="${script_path}/"

run="${script_path_root}../../openssh_server_docker/simple/run.sh"

# https://unix.stackexchange.com/questions/55913/whats-the-easiest-way-to-find-an-unused-local-port
read LOWERPORT UPPERPORT < /proc/sys/net/ipv4/ip_local_port_range
listen_port=$(comm -23 <(seq $LOWERPORT $UPPERPORT | sort) <(ss -Htan | awk '{print $4}' | cut -d':' -f2 | sort -u) | shuf | head -n 1)

export IS_INTERNAL_TEST_OPENSSH_SERVER="1"
export SSH_SERVER_HOST="127.0.0.1"
export SSH_SERVER_PORT="${listen_port}"
export SSH_USERNAME="linuxserver.io"
export SSH_PASSWORD="password"

${run} ${version} ${listen_port} "cd ${script_path_root}..; cargo test -p async-ssh2-lite --features _integration_tests,async-io,tokio -- --nocapture"
${run} ${version} ${listen_port} "cd ${script_path_root}..; cargo test -p async-ssh2-lite --features _integration_tests,_integration_tests_tokio_ext,async-io,tokio -- --nocapture"

################################################ 
# 
# Manual
# 
# In server
# $ sudo vim /etc/ssh/sshd_config
# PubkeyAcceptedAlgorithms +ssh-dss,ssh-rsa
# 
# $ sudo systemctl restart sshd
# 
# In local
# $ SSH_SERVER_HOST=1.1.1.1 SSH_SERVER_PORT=22 SSH_USERNAME=root SSH_PASSWORD=xxxxxx SSH_PRIVATEKEY_PATH=~/.ssh/id_rsa cargo test -p async-ssh2-lite --features _integration_tests,_integration_tests_tokio_ext,async-io,tokio -- --nocapture
# 
################################################ 
