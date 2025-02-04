#!/bin/sh

ip addr add 127.0.0.1/8 dev lo
ip addr add 127.0.0.2/8 dev lo
ip addr add 127.0.0.3/8 dev lo
ip link set dev lo up
socat VSOCK-LISTEN:8888,fork tcp-connect:127.0.0.1:8888,reuseaddr & # for the json rpc server
socat tcp-listen:8889,fork vsock-connect:3:8889,reuseaddr & # for the db
socat VSOCK-LISTEN:8890,fork tcp-connect:127.0.0.3:8890,reuseaddr & # for the websocket

./usr/local/bin/tee-server \
    --server-address=127.0.0.1:8888 \
    --ws-server-url=127.0.0.3:8890 \
    --database-url=postgres://postgres:passport@127.0.0.2:8889/db \
    --circuit-folder=/circuits \
    --zkey-folder=/zkeys \
    --circuit-zkey-map registerSha1Sha256Sha256Rsa655374096=registerSha1Sha256Sha256Rsa655374096.zkey \
    --circuit-zkey-map registerSha256Sha256Sha256EcdsaBrainpoolP256r1=registerSha256Sha256Sha256EcdsaBrainpoolP256r1.zkey \
    --rapidsnark-path=/rapidsnark