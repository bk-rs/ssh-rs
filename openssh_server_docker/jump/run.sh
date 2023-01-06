#!/usr/bin/env bash

set -ex

# ./run.sh version-9.0_p1-r2 2223 2224 "sleep 3"

version="${1:-version-9.0_p1-r2}"
listen_port_bastion=$2
listen_port_intranet=$3
callback=$4

if [ -z "$listen_port_bastion" ]
then
    exit 91
fi
if [ -z "$listen_port_intranet" ]
then
    exit 92
fi
if [ -z "$callback" ]
then
    exit 93
fi

if ps -p $SSH_AGENT_PID > /dev/null
then
    echo "ssh-agent is already running"
else
    echo "require ssh-agent is running"
    exit 96
fi

script_path=$(cd $(dirname $0) ; pwd -P)
script_path_root="${script_path}/"

# 
container_name_bastion="openssh_server_bastion_${listen_port_bastion}"
container_name_intranet="openssh_server_intranet_${listen_port_intranet}"

keys_dir="${script_path_root}keys"
hostname_bastion="openssh_server_bastion"
hostname_intranet="openssh_server_intranet"

chmod 600 "${keys_dir}/id_rsa"
chmod 600 "${keys_dir}/id_dsa"
ssh-add "${keys_dir}/id_rsa"

cleanup() {
    docker stop ${container_name_bastion} ${container_name_intranet}

    ssh-add -d "${keys_dir}/id_rsa"

    sleep 1
}
trap cleanup EXIT

docker run -d --rm --name ${container_name_bastion} \
    -v "${keys_dir}/id_rsa.pub":/pubkeys/id_rsa.pub \
    -v "${script_path_root}../sshd_append_PubkeyAcceptedAlgorithms.sh":/etc/cont-init.d/51-sshd_append_PubkeyAcceptedAlgorithms.sh \
    -v "${script_path_root}../sshd_yes_AllowTcpForwarding.sh":/etc/cont-init.d/51-sshd_yes_AllowTcpForwarding.sh \
    --hostname ${hostname_bastion} \
    -p ${listen_port_bastion}:2222\
    -e PUID=`id -u` \
    -e PGID=`id -g` \
    -e PUBLIC_KEY_DIR=/pubkeys \
    -e USER_NAME=user_bastion \
    linuxserver/openssh-server:${version}

docker run -d --rm --name ${container_name_intranet} \
    -v "${keys_dir}/id_rsa.pub":/pubkeys/id_rsa.pub \
    -v "${script_path_root}../sshd_append_PubkeyAcceptedAlgorithms.sh":/etc/cont-init.d/51-sshd_append_PubkeyAcceptedAlgorithms.sh \
    --hostname ${hostname_intranet} \
    -p ${listen_port_intranet}:2222\
    -e PUID=`id -u` \
    -e PGID=`id -g` \
    -e PUBLIC_KEY_DIR=/pubkeys \
    -e USER_NAME=user_intranet \
    linuxserver/openssh-server:${version}

sleep 1

if [ -x "$(command -v socat)" ]; then
    { echo -e "\r\n"; } | socat TCP4:127.0.0.1:${listen_port_bastion} stdio
    { echo -e "\r\n"; } | socat TCP4:127.0.0.1:${listen_port_intranet} stdio
fi

# cannot skip StrictHostKeyChecking in bastion host
echo "ssh user_intranet@172.17.0.1 -p ${listen_port_intranet} -A -J user_bastion@127.0.0.1:${listen_port_bastion} -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -o ControlMaster=no -v"

echo "ssh user_intranet@172.17.0.1 -p ${listen_port_intranet} -A -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -o ControlMaster=no -o ProxyCommand=\"ssh user_bastion@127.0.0.1 -p ${listen_port_bastion} -W %h:%p -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -o ControlMaster=no\" -v"

# 
echo "callback running..."
bash -c "${callback}"
