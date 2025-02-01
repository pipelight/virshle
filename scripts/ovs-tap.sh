#!/usr/bin/env bash

###################
# OvS create network
#

# Can't set tap device mac with ovs-vsctl
# But it seems to be based on device name.
#
mac="52:3d:2b:1d:dd:24"
brname="vs0"
ifname="tap1"
uuid="test"

# Clean
sudo rm /var/lib/virshle/socket/$uuid.sock

sudo ovs-vsctl \
  -- --if-exists del-port $brname $ifname

# Create tap
sudo ovs-vsctl \
  -- --may-exist add-port $brname $ifname \
  -- set interface $ifname type=tap
sudo macchanger --mac=$mac $ifname
sudo ip link set up dev $ifname


# Create vm
sudo cloud-hypervisor \
    --api-socket /var/lib/virshle/socket/$uuid.sock \
    --kernel /run/cloud-hypervisor/hypervisor-fw \
    --disk path=/home/anon/Iso/nixos.efi.img path=/home/anon/Iso/pipelight-init.img \
    --cpus boot=2 \
    --memory size=512M \
    --console off \
    --serial tty \
    --cmdline "earlyprintk=ttyS0 console=ttyS0 console=hvc0 root=/dev/vda1 rw" \

# ch-remote \
#   --api-socket /var/lib/virshle/socket/$uuid.sock \
#   add-net tap=$ifname
