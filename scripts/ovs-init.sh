#!/usr/bin/env bash

###################
# OvS create network
#

# Can't set tap device mac with ovs-vsctl
# But it seems to be based on device name.
 
ifname="vm1"
brname="br0"
uuid="test"

# Clean
sudo rm /var/lib/virshle/socket/$uuid.sock
sudo rm /tmp/vhost-user1

sudo ovs-vsctl \
  -- --if-exists del-br $brname
sudo ovs-vsctl \
  -- --if-exists del-port $brname $ifname


sudo ovs-vsctl \
  -- add-port $brname $ifname \
  -- set interface $ifname type=dpdkvhostuserclient \
  -- set interface $ifname options:vhost-server-path=/tmp/vhost-user1 \
  -- set interface $ifname options:n_rxq=2

cloud-hypervisor \
    --api-socket /var/lib/virshle/vm/$uuid/ch.sock \
    --kernel /run/cloud-hypervisor/hypervisor-fw \
    --disk path=/home/anon/Iso/nixos.efi.img path=/home/anon/Iso/pipelight-init.img \
    --cpus boot=2 \
    --memory size=512M,hugepages=on,shared=true \
    --console off \
    --serial tty \
    --cmdline "earlyprintk=ttyS0 console=ttyS0 console=hvc0 root=/dev/vda1 rw"
