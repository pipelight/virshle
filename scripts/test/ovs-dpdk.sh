#!/usr/bin/env bash

###################
# OvS create network
#

# Can't set tap device mac with ovs-vsctl
# But it seems to be based on device name.
 
ifname="vm-test"
brname="br0"
uuid="test"

env_dir="/var/lib/virshle/vm/test"

# Clean
sudo rm $env_dir/$uuid.sock
sudo rm /tmp/vhost-user1.sock

# sudo ovs-vsctl \
#   -- --if-exists del-br $brname
sudo ovs-vsctl \
  -- --if-exists del-port $brname $ifname
#
# Create env
sudo mkdir -p ${env_dir}/net
sudo mkdir -p ${env_dir}/disk
cp /home/anon/Iso/nixos.efi.img $env_dir/disk/os

sudo ovs-vsctl \
  -- add-port $brname $ifname \
  -- set interface $ifname type=dpdkvhostuserclient \
  -- set interface $ifname options:vhost-server-path=$env_dir/net/main.sock \
  -- set interface $ifname options:n_rxq=2

cloud-hypervisor \
    --api-socket $env_dir/ch.sock \
    --kernel /run/cloud-hypervisor/hypervisor-fw \
    --disk path=os \
    --cpus boot=1 \
    --memory size=2048M,hugepages=on,shared=true \
    --console off \
    --serial tty \
    --cmdline "earlyprintk=ttyS0 console=ttyS0 console=hvc0 root=/dev/vda1 rw" \
    --net vhost_user=true,socket=/tmp/vhost-user1,num_queues=4,vhost_mode=server
