#!/usr/bin/env bash

set -ex

sed -i "s/^AllowTcpForwarding[\t ].*/AllowTcpForwarding\tyes/" /etc/ssh/sshd_config
