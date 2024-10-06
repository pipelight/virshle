#!/usr/bin/env bash

cloud-hypervisor \
    --bios /run/libvirt/nix-ovmf/OVMF_CODE.fd \
    --console off \
    --serial tty \
    --disk path=/home/anon/Iso/nixos.qcow2 \
    --cmdline "earlyprintk=ttyS0 console=ttyS0" \
    --seccomp true \
    --cpus boot=2 \
    --memory size=4G \
    -vvvv
