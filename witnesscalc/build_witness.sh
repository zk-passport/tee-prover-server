#!/usr/bin/env bash

if [ -z "$1" ] || [ -z "$2" ]; then
    echo "Usage: $0 <circuit name> <path to witnesscalc repo>"
    exit 1
fi

if [[ "$2" != /* && "$2" != ~* ]]; then
    echo "Error: The provided path must be an absolute path."
    exit 1
fi

if [[ "$2" == */ ]]; then
    echo "Error: The provided path must not end with a '/'."
    exit 1
fi

baseName="$1"
witness_calc="$2"

mkdir "${witness_calc}/build_witnesscalc_${baseName}"

cd "${witness_calc}/build_witnesscalc_${baseName}" && cmake .. -DTARGET_PLATFORM=x86_host -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=../package

make ${baseName}

cp ${witness_calc}/src/${baseName}.dat "${witness_calc}/build_witnesscalc_${baseName}/src/${baseName}.dat"