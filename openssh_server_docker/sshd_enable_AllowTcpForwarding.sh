#!/usr/bin/env bash

set -ex

sed "s/^AllowTcpForwarding[\t ].*/AllowTcpForwarding\tyes/" -i /etc/ssh/sshd_config
