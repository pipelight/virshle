tap_if=vmtap0
bridge=vmbr0

sudo mkdir -p /etc/nftables

sudo tee /etc/nftables/masquerade.nft >/dev/null <<EOF
#!/usr/sbin/nft -f

flush ruleset

table ip nat {
    chain postrouting {
        type nat hook postrouting priority 0;
        iifname "$bridge" masquerade
    }
}
EOF

sudo nft -f /etc/nftables/masquerade.nft
sudo ip link add "$bridge" type bridge
sudo ip address add 172.30.0.1/24 dev "$bridge"
sudo ip link set "$bridge" type up
sudo ip link set "$tap_if" master "$bridge"
sudo sysctl -w net.ipv4.conf.all.forwarding=1
sudo sysctl -w net.ipv6.conf.all.forwarding=1

# Guest
# netdev=ens4  # replace as neeeded
# sudo ip link set "$netdev" up
# sudo ip address add 172.30.0.2/24 dev "$netdev"
# sudo ip route add default via 172.30.0.1 dev "$netdev"
# ping 1.1.1.1


# create a bridge
ovs-vsctl add-br ovsbr0 -- set bridge ovsbr0 datapath_type=netdev
# create two DPDK ports and add them to the bridge
ovs-vsctl add-port ovsbr0 vhost-user1 -- set Interface vhost-user1 type=internal options:vhost-server-path=/var/lib/virshle/net/vhost-user1

ovs-vsctl add-port ovsbr0 vhost-user2 -- set Interface vhost-user2 type=internal options:vhost-server-path=/var/lib/virshle/net/vhost-user2

# set the number of rx queues
ovs-vsctl set Interface vhost-user1 options:n_rxq=2
ovs-vsctl set Interface vhost-user2 options:n_rxq=2

# make an access port
ovs-vsctl add-br br0
ovs-vsctl add-port br0 eno1
# ovs-vsctl set port tap0 tag=9
ovs-vsctl add-port br0 tap0 tag=9



