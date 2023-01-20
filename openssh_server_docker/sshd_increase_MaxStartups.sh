#!/usr/bin/env bash

set -ex

sed -i -E "s/^[#]?MaxStartups[\t ].*/MaxStartups\t1000/" /etc/ssh/sshd_config
