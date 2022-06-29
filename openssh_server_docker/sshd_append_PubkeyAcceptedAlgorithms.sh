#!/usr/bin/env bash

set -ex

grep -q -E "^PubkeyAcceptedAlgorithms(\t| )" /etc/ssh/sshd_config \
    && sed "s/^PubkeyAcceptedAlgorithms\t.*/PubkeyAcceptedAlgorithms\t+ssh-dss/" -i /etc/ssh/sshd_config \
    || sed "$ a\PubkeyAcceptedAlgorithms\t+ssh-dss" -i /etc/ssh/sshd_config
