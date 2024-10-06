#!/usr/bin/env bash

# qemu-kvm \
/run/libvirt/nix-emulators/qemu-kvm \
    -name guest=newage_abarai,debug-threads=on \
    -blockdev driver=file,filename=/run/libvirt/nix-ovmf/OVMF_CODE.fd,node-name=libvirt-pflash0-storage,auto-read-only=true,discard=unmap \
    -blockdev node-name=libvirt-pflash0-format,read-only=true,driver=raw,file=libvirt-pflash0-storage \
    -blockdev driver=file,filename=/home/anon/Iso/OVMF_VARS.fd,node-name=libvirt-pflash1-storage,auto-read-only=true,discard=unmap \
    -blockdev node-name=libvirt-pflash1-format,read-only=true,driver=raw,file=libvirt-pflash1-storage \
    -blockdev driver=file,filename=/home/anon/Iso/nixos.qcow2,node-name=libvirt-1-storage,auto-read-only=true,discard=unmap \
    -machine pc-q35-3.0,usb=off,smm=on,dump-guest-core=off,memory-backend=pc.ram,pflash0=libvirt-pflash0-format,pflash1=libvirt-pflash1-format,acpi=on \
    -accel kvm -cpu qemu64,x2apic=on,hypervisor=on,lahf-lm=on,svm=off -m size=4194304k \
    -object memory-backend-ram,id=pc.ram,size=4294967296 \
    -overcommit mem-lock=off -smp 1,sockets=1,cores=1,threads=1 -uuid 6f357084-7c50-48af-9b4d-6afc9bc5e992 \
    -blockdev node-name=libvirt-1-format,read-only=false,driver=qcow2,file=libvirt-1-storage
    # -hda /home/anon/Iso/nixos.qcow2
    #-nographic \

# /run/libvirt/nix-emulators/qemu-kvm \
#   -name guest=gojo_abarai,debug-threads=on \
#   -S \
#   -object {"qom-type":"secret","id":"masterKey0","format":"raw","file":"/var/lib/libvirt/qemu/domain-1-gojo_abarai/master-key.aes"} \
#   -blockdev {"driver":"file","filename":"/run/libvirt/nix-ovmf/OVMF_CODE.fd","node-name":"libvirt-pflash0-storage","auto-read-only":true,"discard":"unmap"} \
#   -blockdev {"node-name":"libvirt-pflash0-format","read-only":true,"driver":"raw","file":"libvirt-pflash0-storage"} \
#   -blockdev {"driver":"file","filename":"/home/anon/Iso/OVMF_VARS.fd","node-name":"libvirt-pflash1-storage","auto-read-only":true,"discard":"unmap"} \
#   -blockdev {"node-name":"libvirt-pflash1-format","read-only":false,"driver":"raw","file":"libvirt-pflash1-storage"} \
#   -machine pc-q35-3.0,usb=off,smm=on,dump-guest-core=off,memory-backend=pc.ram,pflash0=libvirt-pflash0-format,pflash1=libvirt-pflash1-format,acpi=on -accel kvm -cpu qemu64,x2apic=on,hypervisor=on,lahf-lm=on,svm=off -m size=4194304k \
#   -object {"qom-type":"memory-backend-ram","id":"pc.ram","size":4294967296} \
#   -overcommit mem-lock=off -smp 1,sockets=1,cores=1,threads=1 -uuid 6f357084-7c50-48af-9b4d-6afc9bc5e992 -display none -no-user-config -nodefaults -chardev socket,id=charmonitor,fd=35,server=on,wait=off -mon chardev=charmonitor,id=monitor,mode=control -rtc base=utc -no-shutdown -boot strict=on \
#   -device {"driver":"pcie-root-port","port":8,"chassis":1,"id":"pci.1","bus":"pcie.0","multifunction":true,"addr":"0x1"} \
#   -device {"driver":"pcie-root-port","port":9,"chassis":2,"id":"pci.2","bus":"pcie.0","addr":"0x1.0x1"} \
#   -device {"driver":"pcie-root-port","port":10,"chassis":3,"id":"pci.3","bus":"pcie.0","addr":"0x1.0x2"} \
#   -device {"driver":"pcie-root-port","port":11,"chassis":4,"id":"pci.4","bus":"pcie.0","addr":"0x1.0x3"} \
#   -device {"driver":"pcie-root-port","port":12,"chassis":5,"id":"pci.5","bus":"pcie.0","addr":"0x1.0x4"} \
#   -device {"driver":"qemu-xhci","id":"usb","bus":"pci.2","addr":"0x0"} \
#   -blockdev {"driver":"file","filename":"/var/lib/virshle/files/6f357084-7c50-48af-9b4d-6afc9bc5e992_nixos.qcow2","node-name":"libvirt-1-storage","auto-read-only":true,"discard":"unmap"} \
#   -blockdev {"node-name":"libvirt-1-format","read-only":false,"driver":"qcow2","file":"libvirt-1-storage","backing":null} \
#   -device {"driver":"virtio-blk-pci","bus":"pci.3","addr":"0x0","drive":"libvirt-1-format","id":"virtio-disk0","bootindex":1} \
#   -netdev {"type":"tap","fd":"36","vhost":true,"vhostfd":"38","id":"hostnet0"} \
#   -device {"driver":"virtio-net-pci","netdev":"hostnet0","id":"net0","mac":"52:54:00:35:65:7c","bus":"pci.1","addr":"0x0"} \
#   -chardev pty,id=charserial0 -device {"driver":"isa-serial","chardev":"charserial0","id":"serial0","index":0} \
#   -audiodev {"id":"audio1","driver":"none"} \
#   -global ICH9-LPC.noreboot=off -watchdog-action reset -incoming defer \
#   -device {"driver":"virtio-balloon-pci","id":"balloon0","bus":"pci.4","addr":"0x0"} \
#   -sandbox on,obsolete=deny,elevateprivileges=deny,spawn=deny,resourcecontrol=deny -msg timestamp=on
#
