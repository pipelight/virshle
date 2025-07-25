qemu-kvm \
    -nographic \
    -machine q35 \
    -cpu host \
    -smp 2 \
    -m 4G \
    -boot d \
    -hda /home/anon/Iso/nixos.qcow2
