#!/usr/bin/env bash

set -ex

sed -i -E "s/^[#]?MaxStartups[\t ].*/MaxStartups\t30:10:60/" /etc/ssh/sshd_config
