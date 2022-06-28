#!/usr/bin/env bash

set -ex

# ./run.sh 8.8_p1-r1-ls82 2223 2224 "sleep 3"

version="${1:-8.8_p1-r1-ls82}"
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

script_path=$(cd $(dirname $0) ; pwd -P)
script_path_root="${script_path}/"

# 
container_name_bastion="openssh_server_bastion_${listen_port_bastion}"
container_name_intranet="openssh_server_intranet_${listen_port_intranet}"

config_dir_bastion="${script_path_root}config_bastion"
config_dir_intranet="${script_path_root}config_intranet"
keys_dir="${script_path_root}keys"
hostname_bastion="openssh_server_bastion"
hostname_intranet="openssh_server_intranet"

cleanup() {
    docker stop ${container_name_bastion} ${container_name_intranet}

    sleep 1
}
trap cleanup EXIT

docker run -d --rm --name ${container_name_bastion} \
    -v "${keys_dir}/id_rsa.pub":/pubkeys/id_rsa.pub \
    -v "${config_dir_bastion}":/config \
    --hostname ${hostname_bastion} \
    -p ${listen_port_bastion}:2222\
    -e PUID=`id -u` \
    -e PGID=`id -g` \
    -e PUBLIC_KEY_DIR=/pubkeys \
    -e SUDO_ACCESS=true \
    -e USER_NAME=user_bastion \
    linuxserver/openssh-server:${version}

docker run -d --rm --name ${container_name_intranet} \
    -v "${keys_dir}/id_rsa.pub":/pubkeys/id_rsa.pub \
    -v "${config_dir_intranet}":/config \
    --hostname ${hostname_intranet} \
    -p ${listen_port_intranet}:2222\
    -e PUID=`id -u` \
    -e PGID=`id -g` \
    -e PUBLIC_KEY_DIR=/pubkeys \
    -e SUDO_ACCESS=true \
    -e USER_NAME=user_intranet \
    linuxserver/openssh-server:${version}

sleep 1

if [ -x "$(command -v socat)" ]; then
    { echo -e "\r\n"; } | socat TCP4:127.0.0.1:${listen_port_bastion} stdio
    { echo -e "\r\n"; } | socat TCP4:127.0.0.1:${listen_port_intranet} stdio
fi

echo "ssh user_intranet@127.0.0.1 -p ${listen_port_intranet} -A -J user_bastion@127.0.0.1:${listen_port_bastion} -i keys/id_rsa -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=off -o ProxyCommand=\"ssh -i keys/id_rsa -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=off\" -v"

# 
echo "callback running..."
bash -c "${callback}"
