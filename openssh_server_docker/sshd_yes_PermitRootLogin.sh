#!/usr/bin/env bash

set -ex

sed -i -E "s/^[#]?PermitRootLogin[\t ].*/PermitRootLogin\tyes/" /etc/ssh/sshd_config
