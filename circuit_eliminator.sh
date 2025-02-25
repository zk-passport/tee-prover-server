#!/bin/bash

# Define the arrays
register_circuits=("register_sha512_sha512_sha512_ecdsa_brainpoolP512r1" "register_sha384_sha384_sha384_ecdsa_brainpoolP512r1")
dsc_circuits=("dsc_sha384_ecdsa_brainpoolP512r1" "dsc_sha512_ecdsa_brainpoolP512r1")
disclose_circuits=()

proof_type=$1

case "$proof_type" in
  "register")
    circuits=("${register_circuits[@]}")
    ;;
  "dsc")
    circuits=("${dsc_circuits[@]}")
    ;;
  "disclose")
    circuits=("${disclose_circuits[@]}")
    ;;
  *)
    echo "Invalid proof type: $proof_type"
    exit 1
    ;;
esac

for circuit in "${circuits[@]}"; do
  echo /circuits/${circuit}_cpp
  rm -rf "/circuits/${circuit}_cpp"
  rm -rf "/zkeys/${circuit}.zkey"
done

echo "Cleanup complete for $proof_type circuits."