## Init

```
chmod 600 keys/{id_rsa,id_dsa}

sudo rm -rf config/{.ssh,custom-cont-init.d,custom-services.d,logs,ssh_host_keys,.bash_history}

./run.sh 8.8_p1-r1-ls82 2222 "sleep 3"

sudo cp ../sshd_enable_dsa.sh config/custom-cont-init.d/

./run.sh 8.8_p1-r1-ls82 2222 "sleep 300"
```
