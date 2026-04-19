#!/usr/bin/env bash

vm_uuid="22fd9d87-e4bf-4cd6-96da-7b4c9ae68df3"

sudo chown -R anon:users /var/lib/virshle/vm/$vm_uuid
rm -rf /var/lib/virshle/vm/$vm_uuid/ch.*
cloud-hypervisor \
  --api-socket /var/lib/virshle/vm/$vm_uuid/ch.sock \
  --cmdline "earlyprintk=ttyS0 console=ttyS0 console=hvc0" \
  --disk path=/var/lib/virshle/vm/$vm_uuid/disk/nixos.xxs.efi.img \
  --cpus boot=2 \
  --memory size=2048M \
  --kernel=/run/cloud-hypervisor/hypervisor-fw \
  --watchdog \
  -v

# rm -f ~/Iso/ch.sock
# cloud-hypervisor \
#   --api-socket ~/Iso/ch.sock \
#   --cmdline "earlyprintk=ttyS0 console=ttyS0 console=hvc0 root=/dev/vda2 rw" \
#   --disk path=~/Iso/disfunctional.xs.efi.raw \
#   --cpus boot=2 \
#   --memory size=2048M \
#   --kernel=/run/cloud-hypervisor/hypervisor-fw \
#   --watchdog \
#   -v
