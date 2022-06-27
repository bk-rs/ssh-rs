#!/usr/bin/env bash

set -ex

# ./run.sh 8.8_p1-r1-ls82 2222 "sleep 3"

version="${1:-8.8_p1-r1-ls82}"
listen_port=$2
callback=$3

if [ -z "$listen_port" ]
then
    exit 91
fi
if [ -z "$callback" ]
then
    exit 92
fi

script_path=$(cd $(dirname $0) ; pwd -P)
script_path_root="${script_path}/"

# 
container_name="openssh_server_${listen_port}"

config_dir="${script_path_root}config"
keys_dir="${script_path_root}keys"
hostname="openssh_server_${listen_port}"

cleanup() {
    docker stop ${container_name}

    sleep 1
}
trap cleanup EXIT

docker run -d --rm --name ${container_name} \
    -v "${keys_dir}/id_rsa.pub":/pubkeys/id_rsa.pub \
    -v "${keys_dir}/id_dsa.pub":/pubkeys/id_dsa.pub \
    -v "${config_dir}":/config \
    --hostname ${hostname} \
    -p ${listen_port}:2222\
    -e PUID=`id -u` \
    -e PGID=`id -g` \
    -e PUBLIC_KEY_DIR=/pubkeys \
    -e SUDO_ACCESS=true \
    -e PASSWORD_ACCESS=true \
    -e USER_NAME=linuxserver.io \
    -e USER_PASSWORD=password \
    linuxserver/openssh-server:${version}

sleep 1

if [ -x "$(command -v socat)" ]; then
    # https://www.compose.com/articles/how-to-talk-raw-redis/
    # https://gist.github.com/eeddaann/6e2b70e36f7586a556487f663b97760e
    { echo -e "\r\n"; } | socat TCP4:127.0.0.1:${listen_port} stdio
fi

echo "ssh linuxserver.io@127.0.0.1 -p ${listen_port} -i keys/id_rsa -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=off -v"
echo "ssh linuxserver.io@127.0.0.1 -p ${listen_port} -i keys/id_dsa -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=off -o HostKeyAlgorithms=+ssh-dss -o PubkeyAcceptedAlgorithms=+ssh-dss -v"
echo "ssh linuxserver.io@127.0.0.1 -p ${listen_port} -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=off -o PreferredAuthentications=password -o PubkeyAuthentication=no -v"

# 
echo "callback running..."
bash -c "${callback}"
