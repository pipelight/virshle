#!/usr/bin/env bash
# minoru-kurosaki
# vm_uuid="0fea3bf6-862c-4c04-b9d5-940c2810305f"
# 662
# vm_uuid="0d1aa5c2-d114-4492-ac40-f56c32812df4"

# karin-linus
# vm_uuid="95c2e1c1-67ef-4ca7-b0c7-a77a09c9d0d5"
#
# 666 vps-web clone
vm_uuid="8abb6671-0d6d-45db-8e77-c712cee5df11"

# sudo chown -R anon:users /var/lib/virshle/vm/$vm_uuid
rm -rf /var/lib/virshle/vm/$vm_uuid/ch.*
cloud-hypervisor \
  --api-socket /var/lib/virshle/vm/$vm_uuid/ch.sock \
  --cmdline "earlyprintk=ttyS0 console=ttyS0 console=hvc0 root=/dev/vda2 rw" \
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
