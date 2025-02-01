#!/usr/bin/env bash

set -x

# Host network adapter to bridge the guest onto
host_net="eno1"

###################
# Tap device
sudo ip link del macvtap0  2> /dev/null
rm /var/lib/virshle/socket/test.sock 2> /dev/null
# The MAC address must be attached to the macvtap and be used inside the guest
mac="c2:67:4f:53:29:cb"
# Create the macvtap0 as a new virtual MAC associated with the host network
sudo ip link add link "$host_net" name macvtap0 type macvtap
sudo ip link set macvtap0 address "$mac" up
sudo ip link show macvtap0


###################
# Bridge interface
# mac="c2:67:4f:53:29:a1"
# sudo ip link add link "$host_net" name tap0 type tap

###################
# Cloud hypervisor socket
uuid="test"

# A new character device is created for this interface
tapindex=$(< /sys/class/net/macvtap0/ifindex)
tapdevice="/dev/tap$tapindex"

# Ensure that we can access this device
sudo chown "$UID:$UID" "$tapdevice"

cloud-hypervisor \
    --api-socket /var/lib/virshle/socket/$uuid.sock \
    --kernel /run/cloud-hypervisor/hypervisor-fw \
    --console off \
    --serial tty \
    --disk path=/home/anon/Iso/nixos.efi.img path=/home/anon/Iso/pipelight-init.img \
    --cpus boot=2 \
    --memory size=512M \
    --cmdline "earlyprintk=ttyS0 console=ttyS0 console=hvc0 root=/dev/vda1 rw" \
    --net mac=$mac,fd=3 3<>$"$tapdevice"
    
   
# cloud-hypervisor \
#     --api-socket /var/lib/virshle/socket/$uuid.sock \
#     --kernel /run/cloud-hypervisor/hypervisor-fw \
#     --console off \
#     --serial tty \
#     --disk path=/home/anon/Iso/nixos.img path=/home/anon/Iso/pipelight-init.img \
#     --cpus boot=4 \
#     --memory size=512M,hugepages=on,shared=true \
#     --cmdline "earlyprintk=ttyS0 console=ttyS0 console=hvc0 root=/dev/vda1 rw" \
#     --net mac=$mac,fd=3 3<>$"$tapdevice"
