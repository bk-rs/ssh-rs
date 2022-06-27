## Init

```
sudo rm -rf config/{.ssh,custom-cont-init.d,custom-services.d,logs,ssh_host_keys,.bash_history}

./run.sh 8.8_p1-r1-ls82 2222 "sleep 3"

sudo cp edit_sshd_config.ssh config/custom-cont-init.d

./run.sh 8.8_p1-r1-ls82 2222 "sleep 300"
```
