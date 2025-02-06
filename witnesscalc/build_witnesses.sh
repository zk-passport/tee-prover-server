#!/usr/bin/env bash

# run from root of the repo
if [ -z "$1" ]; then
    echo "Usage: $0 <path to open witnesscalc>"
    exit 1
fi
# throw an error if the path is not an absolute path
if [[ "$1" != /* && "$1" != ~* ]]; then
    echo "Error: The provided path must be an absolute path."
    exit 1
fi

# it should not end with a /
if [[ "$1" == */ ]]; then
    echo "Error: The provided path must not end with a '/'."
    exit 1
fi
path="$1"

snake_to_camel() {
    local input="$1"
    local result

    result=$(echo "$input" | awk -F_ '{printf "%s", tolower(substr($1,1,1)) substr($1,2)}')
    result+=$(echo "$input" | awk -F_ '{for (i=2; i<=NF; i++) printf "%s", toupper(substr($i,1,1)) substr($i,2)}')

    echo "$result"
}

circuits=( ["0"]="register_sha1_sha256_sha256_rsa_65537_4096" ["1"]="register_sha256_sha256_sha256_ecdsa_brainpoolP256r1")

for circuit in "${circuits[@]}"; do
    type=${circuit%%_*}
    circuit_name=$(snake_to_camel "$circuit")

    ./build_witness.sh $circuit_name $path
done