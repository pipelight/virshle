#!/usr/bin/env -S bash

# set -euo pipefail

showUsage() {
  cat <<EOF
Usage: $0 [OPTIONS]
  --id INT                 Specify the VM id.
EOF
}

# Initiate global vars.
vm_id=
vm_uuid=

parseArgs() {
  [[ $# -eq 0 ]] && {
    showUsage
    exit 1
  }
  while [[ $# -gt 0 ]]; do
    case "$1" in
    -d | --debug)
      # Print commands
      set -x
      ;;
    -h | --help)
      showUsage
      exit 0
      ;;
    --id)
      vm_id=$2
      vm_uuid=$(getUuid $vm_id)
      mountDisk $vm_uuid
      ;;
    --uuid)
      vm_uuid=$2
      mountDisk $vm_uuid
    esac
    shift
  done
}

# Get vm uuid from its id.
getUuid() {
  vm_id=$1

  vm_definition=$(virshle vm ls \
    --id $vm_id \
    --json)
  vm_uuid=$(echo $vm_definition | jq -r ".uuid")

  echo $vm_uuid
}

# Mount Vm disk
mountDisk() {

  # Make disk path
  vm_uuid=$1

  vms_dir="/var/lib/virshle/vm"
  disk_rel_path="disk/nixos.xxs.efi.img"
  disk_path="$vms_dir/$vm_uuid/$disk_rel_path"

  device="/dev/loop10"
  efi_partition=$device"p1"
  root_partition=$device"p2"

  sudo losetup --partscan $device $disk_path

  sudo fsck.vfat -fy $efi_partition
  sudo fsck.ext4 -fy $root_partition

  # sudo losetup -d $device
  # sudo losetup --partscan $device $disk_path
}

main() {
  parseArgs $@
  exit 0;
}

main $@
