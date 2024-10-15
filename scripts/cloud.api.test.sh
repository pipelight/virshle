#!/usr/bin/env bash
set -x

uuid="b30458d1-7c7f-4d06-acc2-159e43892e87"

# curl --unix-socket /var/lib/virshle/socket/$uuid.sock -i \
#      -X GET 'http://localhost/api/v1/vm.info' \
#      -H 'Accept: application/json'
#
curl --unix-socket /var/lib/virshle/socket/$uuid.sock -i \
     -X GET 'http://localhost/api/v1/vm.info' \
     -H 'Accept: application/json' 

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
