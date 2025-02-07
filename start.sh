#!/bin/sh

ip addr add 127.0.0.1/8 dev lo
ip addr add 127.0.0.2/8 dev lo
ip link set dev lo up
socat VSOCK-LISTEN:8888,fork tcp-connect:127.0.0.1:8888,reuseaddr & # for the json rpc server
socat tcp-listen:8889,fork vsock-connect:3:8889,reuseaddr & # for the db
socat tcp-listen:8000,fork vsock-connect:3:8000,reuseaddr & # for s3

# Download the zkeys from s3
cd /
wget --bind-address=127.0.0.1:8000 --continue --no-check-certificate https://self-protocol.s3.eu-west-1.amazonaws.com/zkeys/zkeys.tar.zst
tar -I zstd -xvf zkeys.tar.zst

/usr/local/bin/tee-server \
    --server-address=127.0.0.1:8888 \
    --database-url=postgres://postgres:passport@127.0.0.2:8889/openpassport \
    --circuit-folder=/circuits \
    --zkey-folder=/zkeys \
    --circuit-zkey-map register_sha1_sha256_sha256_rsa_65537_4096=register_sha1_sha256_sha256_rsa_65537_4096.zkey \
    --circuit-zkey-map register_sha256_sha256_sha256_ecdsa_brainpoolP256r1=register_sha256_sha256_sha256_ecdsa_brainpoolP256r1.zkey \
    --rapidsnark-path=/rapidsnark

# For running locally
# cargo run -- \
#     --server-address=127.0.0.1:8888 \
#     --database-url=postgres://postgres:mysecretpassword@127.0.0.1:5433/db \
#     --circuit-folder=./circuits \
#     --zkey-folder=./zkeys \
#     --circuit-zkey-map register_sha1_sha256_sha256_rsa_65537_4096=register_sha1_sha256_sha256_rsa_65537_4096.zkey \
#     --circuit-zkey-map register_sha256_sha256_sha256_ecdsa_brainpoolP256r1=register_sha256_sha256_sha256_ecdsa_brainpoolP256r1.zkey \
#     --rapidsnark-path=./rapidsnark
