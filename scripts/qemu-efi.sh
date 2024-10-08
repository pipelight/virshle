#!/usr/bin/env bash

# Minimal qemu command to boot nixos image(nixos-generators)
# with efi support.

qemu-kvm \
    -nographic \
    -machine q35 \
    -cpu host \
    -smp 2 \
    -m 4G \
    -boot d \
    -drive if=pflash,format=raw,readonly=on,file=/run/libvirt/nix-ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=/home/anon/Iso/OVMF_VARS.fd \
    -hda /home/anon/Iso/nixos.qcow2
    
qemu-kvm \
    -nographic \
    -machine q35 \
    -cpu host \
    -smp 2 \
    -m 4G \
    -boot d \
    -drive if=pflash,format=raw,readonly=on,file=/run/libvirt/nix-ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=/home/anon/Iso/OVMF_VARS.fd \
    -hda /home/anon/Iso/nixos.qcow2
    
