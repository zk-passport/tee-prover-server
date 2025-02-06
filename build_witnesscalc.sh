#!/usr/bin/env bash

# run from root of the repo
if [ -z "$1" ] || [ -z "$2" ] || [ ! -d "$2" ]; then
    echo "Usage: $0 <proof type (register / dsc / disclose)> <path to openpassport repo> <output folder path>"
    exit 1
fi

PROOF_TYPE=$1
if [ "$PROOF_TYPE" != "register" ] && [ "$PROOF_TYPE" != "dsc" ] && [ "$PROOF_TYPE" != "disclose" ]; then
    echo "Invalid proof type: $PROOF_TYPE"
    exit 1
fi

if [[ "$2" != /* && "$2" != ~* ]] || [[ "$3" != /* && "$3" != ~* ]]; then
    echo "Error: The provided path must be an absolute path."
    exit 1
fi

# it should not end with a /
if [[ "$2" == */ ]] || [[ "$3" == */ ]]; then
    echo "Error: The provided path must not end with a '/'."
    exit 1
fi

openpassportpath="$2"
basepath="$2/circuits/circuits/$PROOF_TYPE/instances"
output="$3"

allowed_circuits=(
    "register_sha256_sha256_sha256_ecdsa_brainpoolP256r1.circom"
    "register_sha1_sha256_sha256_rsa_65537_4096.circom"
)

pids=() 
for file in "$basepath"/*.circom; do
    filename=$(basename "$file")

    if [[ ! " ${allowed_circuits[@]} " =~ " $filename " ]]; then
        echo "Skipping $filename (not in allowed circuits)"
        continue
    fi 

    filepath=$basepath/$filename 
    circom_pid=$!
    circuit_name="${filename%.*}"
    (
        circom $filepath \
        -l "$openpassportpath/circuits/node_modules" \
        -l "$openpassportpath/circuits/node_modules/@zk-kit/binary-merkle-root.circom/src" \
        -l "$openpassportpath/circuits/node_modules/circomlib/circuits" \
        --O1 -c --output $output && \
        cd $output/${circuit_name}_cpp && \
        make 
    ) & 
    pids+=($!)
done

echo "Waiting for all circuits to compile..."
wait "${pids[@]}"
echo "All circuits compiled successfully!"
