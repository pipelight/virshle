#!/usr/bin/env bash

cloud-hypervisor \
    --api-socket /var/lib/virshle/socket/uuid.sock \
    --kernel /run/cloud-hypervisor/hypervisor-fw \
    --console off \
    --serial tty \
    --disk path=/home/anon/Iso/nixos.efi.qcow2 \
    --cpus boot=2 \
    --memory size=4G
