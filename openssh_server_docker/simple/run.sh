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
container_name="openssh_server_${listen_port}"

keys_dir="${script_path_root}keys"
hostname="openssh_server"

chmod 600 "${keys_dir}/id_rsa"
chmod 600 "${keys_dir}/id_dsa"
ssh-add "${keys_dir}/id_rsa"
ssh-add "${keys_dir}/id_dsa"

cleanup() {
    docker logs ${container_name} -n 10
    docker stop ${container_name}

    ssh-add -d "${keys_dir}/id_rsa"
    ssh-add -d "${keys_dir}/id_dsa"

    sleep 1
}
trap cleanup EXIT

# should set PermitRootLogin to yes, because maybe `id -u` is 0
docker run -d --rm --name ${container_name} \
    -v "${keys_dir}/id_rsa.pub":/pubkeys/id_rsa.pub \
    -v "${keys_dir}/id_dsa.pub":/pubkeys/id_dsa.pub \
    -v "${script_path_root}../sshd_append_PubkeyAcceptedAlgorithms.sh":/etc/cont-init.d/51-sshd_append_PubkeyAcceptedAlgorithms.sh \
    -v "${script_path_root}../sshd_yes_AllowTcpForwarding.sh":/etc/cont-init.d/51-sshd_yes_AllowTcpForwarding.sh \
    -v "${script_path_root}../sshd_yes_PermitRootLogin.sh":/etc/cont-init.d/51-sshd_yes_PermitRootLogin.sh \
    -v "${script_path_root}../sshd_increase_MaxSessions.sh":/etc/cont-init.d/51-sshd_increase_MaxSessions.sh \
    -v "${script_path_root}../sshd_increase_MaxStartups.sh":/etc/cont-init.d/51-sshd_increase_MaxStartups.sh \
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
    { echo -e "\r\n"; } | socat TCP4:127.0.0.1:${listen_port} stdio
fi

echo "ssh linuxserver.io@127.0.0.1 -p ${listen_port} -i keys/id_rsa -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -o ControlMaster=no -v"
echo "ssh linuxserver.io@127.0.0.1 -p ${listen_port} -i keys/id_dsa -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -o ControlMaster=no -o HostKeyAlgorithms=+ssh-dss -o PubkeyAcceptedAlgorithms=+ssh-dss -v"
echo "ssh linuxserver.io@127.0.0.1 -p ${listen_port} -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -o ControlMaster=no -o PreferredAuthentications=password -o PubkeyAuthentication=no -v"

# 
echo "callback running..."
bash -c "${callback}"
