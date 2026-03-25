# minoru-kurosaki
vm_uuid="0fea3bf6-862c-4c04-b9d5-940c2810305f"

# karin-linus
# vm_uuid="95c2e1c1-67ef-4ca7-b0c7-a77a09c9d0d5"

rm -rf /var/lib/virshle/vm/$vm_uuid/ch.sock
cloud-hypervisor \
  --api-socket /var/lib/virshle/vm/$vm_uuid/ch.sock \
  --cmdline "earlyprintk=ttyS0 console=ttyS0 console=hvc0 root=/dev/vda1 rw" \
  --disk path=/var/lib/virshle/vm/$vm_uuid/disk/os \
  --cpus boot=1 \
  --memory size=1024M \
  --kernel=/run/cloud-hypervisor/hypervisor-fw
