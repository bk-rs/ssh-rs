#!/usr/bin/env bash

set -ex

sed -i -E "s/^[#]?MaxSessions[\t ].*/MaxSessions\t10/" /etc/ssh/sshd_config
