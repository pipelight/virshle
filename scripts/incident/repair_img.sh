#!/usr/bin/env -S bash

set -euo pipefail

showUsage() {
  cat <<EOF
Usage: $0 [OPTIONS]
  --id INT                 Specify the VM id.
EOF
}

# Initiate global vars.
vm_id=

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
      # Print commands
      vm_id=$2
      mountDisk $vm_id
      ;;
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
  vm_uuid=$(echo $vm_definition | jq ".uuid")

  echo $vm_uuid
}

# Mount Vm disk
mountDisk() {
  vm_id=$1
  vm_uuid=$(getUuid $vm_id)

  virshle_state_dir="/var/lib/virshle/"
  disk_rel_path="disk/os"
  disk_path="$virshle_state_dir/$vm_uuid/$disk_rel_path"
}

main() {
  parseArgs $@
  exit 0;
}

main $@
