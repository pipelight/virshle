#!/usr/bin/env bash

set -x

# Set globals
uuid="test"
mac="76:99:00:d8:28:e2"

# mac="52:54:20:11:C5:02"
# mac="c2:67:4f:53:29:cb"

# Clean previous
# rm /tmp/ch.sock
# rm /var/lib/virshle/net/vhost-user3

# sudo ovs-vsctl del-port vhost-user3


# Host network adapter to bridge the guest onto
# host_net="eno1"
# host_mac="b4:2e:99:4b:cb:20"

# Create the macvtap0 as a new virtual MAC associated with the host network
# sudo ip link add link "$host_net" name macvtap0 type macvtap
# sudo ip link set macvtap0 address "$mac" up
# sudo ip link show macvtap0

# A new character device is created for this interface
# tapindex=$(< /sys/class/net/macvtap0/ifindex)
# tapdevice="/dev/tap$tapindex"

# sudo chown "$UID:$UID" "$tapdevice"
# sudo ovs-vsctl add-port ovsbr1 vhost-user3 -- set Interface vhost-user3 type=dpdkvhostuserclient options:vhost-server-path=/var/lib/virshle/net/vhost-user3
# sudo ovs-vsctl add-port ovsbr1 vhost-user3 -- set Interface vhost-user3 type=internal options:vhost-server-path=/var/lib/virshle/net/vhost-user3

# sudo ovs-vsctl set Interface vhost-user3 options:n_rxq=2


cloud-hypervisor \
    --api-socket /tmp/ch.sock \
    --kernel /run/cloud-hypervisor/hypervisor-fw \
    --console off \
    --serial tty \
    --disk path=/home/anon/Iso/nixos.test.img \
    --cpus boot=1 \
    --memory size=2048M \
    --cmdline "earlyprintk=ttyS0 console=ttyS0 root=/dev/vda1 rw" \

    # --net mac=$mac,vhost_user=true,socket=/var/lib/virshle/net/vhost-user3,num_queues=4,vhost_mode=server

