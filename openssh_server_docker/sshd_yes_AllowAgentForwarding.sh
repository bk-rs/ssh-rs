#!/usr/bin/env bash

set -ex

sed -i -E "s/^[#]?AllowAgentForwarding[\t ].*/AllowAgentForwarding\tyes/" /etc/ssh/sshd_config
