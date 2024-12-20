#!/usr/bin/env bash
# set -x

uuid="test"

curl -s --unix-socket /var/lib/virshle/socket/$uuid.sock \
     -X GET 'http://localhost/api/v1/vm.info' \
     -H 'Accept: application/json' | jq 

# curl --unix-socket /var/lib/virshle/socket/uuid.sock -i \
#      -X PUT 'http://localhost/api/v1/vm.boot' \
#      -H 'Accept: application/json'
#
# curl --unix-socket /var/lib/virshle/socket/$(uuid).sock -i \
#      -X PUT 'http://localhost/api/v1/vm.shutdown' \
#      -H 'Accept: application/json'

# curl --unix-socket /var/lib/virshle/socket/uuid.sock -i \
#      -X PUT 'http://localhost/api/v1/vm.delete' \
#      -H 'Accept: application/json'
