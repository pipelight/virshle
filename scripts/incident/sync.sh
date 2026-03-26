#!/usr/bin/env -S bash

set -euo pipefail

showUsage() {
  cat <<EOF
Usage: $0 [OPTIONS]
  --peer MODE                 Specify the peer to sync with.
EOF
}

# Initiate global vars.
remote=

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
    --peer)
      # Print commands
      remote=$2
      syncFiles $remote
      ;;
    esac
    shift
  done
}


syncFiles() {

  printf "\nSyncing files\n"
  ssh $remote -C "mkdir -p ~/Test"
  rsync --progress ./* $remote:Test/

  # Confirm
  printf "\nListing remote directory\n"
  ssh $remote -C "eza --tree -L2 ~/Test"
}

main() {
  parseArgs $@
  exit 0;
}

main $@
