#!/usr/bin/env bash

set -ex

sed -i -E "s/^[#]?AllowTcpForwarding[\t ].*/AllowTcpForwarding\tyes/" /etc/ssh/sshd_config
