#!/usr/bin/env bash

curl --unix-socket /var/lib/virshle/virshle.sock -i \
     -X GET 'http://localhost/' \
     -H 'Accept: application/json'

curl --unix-socket /var/lib/virshle/virshle.sock -i \
     -X GET 'http://localhost/vm/list' \
     -H 'Accept: application/json'
