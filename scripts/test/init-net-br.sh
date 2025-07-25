#!/usr/bin/env bash

set -x

brname="br0"

# Create dpdk port
sudo ovs-vsctl add-br $brname \
  -- set bridge $brname datapath_type=netdev

# Create patch cable 1/2
sudo ovs-vsctl \
  -- --may-exist add-port vs0 patch_vs0br0 \
  -- set interface patch_vs0br0 type=patch \
  -- set interface patch_vs0br0 options:peer=patch_br0vs0 \


# Create patch cable 2/2
sudo ovs-vsctl \
  -- --may-exist add-port $brname patch_br0vs0 \
  -- set interface patch_br0vs0 type=patch \
  -- set interface patch_br0vs0 options:peer=patch_vs0br0 \

