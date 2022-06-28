## Init

```
chmod 600 keys/{id_rsa,id_dsa}

sudo rm -rf config_bastion/{.ssh,custom-cont-init.d,custom-services.d,logs,ssh_host_keys,.bash_history}
sudo rm -rf config_intranet/{.ssh,custom-cont-init.d,custom-services.d,logs,ssh_host_keys,.bash_history}

./run.sh 8.8_p1-r1-ls82 2223 2224 "sleep 3"

./run.sh 8.8_p1-r1-ls82 2223 2224 "sleep 300"
```
