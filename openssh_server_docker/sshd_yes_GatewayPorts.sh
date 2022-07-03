#!/usr/bin/env bash

set -ex

sed -i -E "s/^[#]?GatewayPorts[\t ].*/GatewayPorts\tyes/" /etc/ssh/sshd_config
