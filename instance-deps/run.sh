#!/bin/bash -e

/usr/bin/socat tcp-listen:8888,fork,reuseaddr vsock-connect:7:8888 & # for the rpc server
/usr/bin/socat vsock-listen:8889,fork,reuseaddr TCP4:mysql.mysql:5432 & # for the db

EIF_PATH=/home/tee-server.eif
ENCLAVE_CPU_COUNT=6
ENCLAVE_MEMORY_SIZE=23500

exec nitro-cli run-enclave --enclave-cid=7 --cpu-count $ENCLAVE_CPU_COUNT --memory $ENCLAVE_MEMORY_SIZE --eif-path $EIF_PATH --debug-mode --attach-console
