#!/bin/bash

# In /CMkaeLists.txt move the new circuits above the "fr" line

if [ -z "$1" ] || [ -z "$2" ] || [ -z "$3" ]; then
    echo "Usage: $0 <circuit name> <header name> <path to open witnesscalc>"
    exit 1
fi

circuit="$1"
header="$2"
witness_calc="$3"

cmake_file="CMakeLists.txt"
content=$(<"$cmake_file")

install_targets_start=$(echo "$content" | grep -n "install(TARGETS" | cut -d: -f1)
install_targets_end=$(echo "$content" | sed -n "$install_targets_start,\$p" | grep -n ")" | head -n 1 | cut -d: -f1)
install_targets_end=$((install_targets_start + install_targets_end - 1))

install_files_start_3=$(echo "$content" | grep -n "install(FILES" | sed -n '2p' | cut -d: -f1)
install_files_end_3=$(echo "$content" | sed -n "$install_files_start_3,\$p" | grep -n ")" | head -n 1 | cut -d: -f1)
install_files_end_3=$((install_files_start_3 + install_files_end_3 - 1 + 3)) # + 3 for the new lines added my targets

install_files_start_last=$(echo "$content" | grep -n "install(FILES" | tail -n 1 | cut -d: -f1)
install_files_end_last=$(echo "$content" | sed -n "$install_files_start_last,\$p" | grep -n ")" | head -n 1 | cut -d: -f1)
install_files_end_last=$((install_files_start_last + install_files_end_last - 1 + 3 + 1)) # + 1 for the new line added my .dat file

target_lines="    ${circuit}\n"
target_lines+="    witnesscalc_${circuit}\n"
target_lines+="    witnesscalc_${circuit}Static"

data_files=""
data_files+="    src/${circuit}.dat"

header_files=""
header_files+="    src/witnesscalc_${circuit}.h"

content=$(echo "$content" | awk -v targets="$target_lines" '
{
    if ($0 ~ /^ *fr *$/) {
        print targets;
    }
    print $0;
}')

content_modified=$(echo "$content")
content_modified=$(echo "$content_modified" | sed "${install_files_end_3}i\\$data_files")
content_modified=$(echo "$content_modified" | sed "${install_files_end_last}i\\$header_files")

echo "$content_modified" > "$cmake_file"

cpp_header_content='
#ifndef WITNESSCALC_PROVESHA1SHA1SHA1RSA655374096_H
#define WITNESSCALC_PROVESHA1SHA1SHA1RSA655374096_H


#ifdef __cplusplus
extern "C" {
#endif

#define WITNESSCALC_OK                  0x0
#define WITNESSCALC_ERROR               0x1
#define WITNESSCALC_ERROR_SHORT_BUFFER  0x2

/**
 *
 * @return error code:
 *         WITNESSCALC_OK - in case of success.
 *         WITNESSCALC_ERROR - in case of an error.
 *
 * On success wtns_buffer is filled with witness data and
 * wtns_size contains the number bytes copied to wtns_buffer.
 *
 * If wtns_buffer is too small then the function returns WITNESSCALC_ERROR_SHORT_BUFFER
 * and the minimum size for wtns_buffer in wtns_size.
 *
 */

int
witnesscalc_proveSha1Sha1Sha1Rsa655374096(
    const char *circuit_buffer,  unsigned long  circuit_size,
    const char *json_buffer,     unsigned long  json_size,
    char       *wtns_buffer,     unsigned long *wtns_size,
    char       *error_msg,       unsigned long  error_msg_maxsize);

#ifdef __cplusplus
}
#endif


#endif // WITNESSCALC_PROVESHA1SHA1SHA1RSA655374096_H
'

cpp_content='
#include "witnesscalc_proveSha1Sha1Sha1Rsa655374096.h"
#include "witnesscalc.h"

int
witnesscalc_proveSha1Sha1Sha1Rsa655374096(
    const char *circuit_buffer,  unsigned long  circuit_size,
    const char *json_buffer,     unsigned long  json_size,
    char       *wtns_buffer,     unsigned long *wtns_size,
    char       *error_msg,       unsigned long  error_msg_maxsize)
{
    return CIRCUIT_NAME::witnesscalc(circuit_buffer, circuit_size,
                       json_buffer,   json_size,
                       wtns_buffer,   wtns_size,
                       error_msg,     error_msg_maxsize);
}
'


touch $witness_calc/src/witnesscalc_${circuit}.h
touch $witness_calc/src/witnesscalc_${circuit}.cpp

echo "$cpp_header_content" > $witness_calc/src/witnesscalc_${circuit}.h
echo "$cpp_content" > $witness_calc/src/witnesscalc_${circuit}.cpp

sed -i "s/WITNESSCALC_PROVESHA1SHA1SHA1RSA655374096_H/WITNESSCALC_${header^^}_H/g" $witness_calc/src/witnesscalc_${circuit}.h
sed -i "s/witnesscalc_proveSha1Sha1Sha1Rsa655374096/witnesscalc_${circuit}/g" $witness_calc/src/witnesscalc_${circuit}.h
sed -i "s/witnesscalc_proveSha1Sha1Sha1Rsa655374096/witnesscalc_${circuit}/g" $witness_calc/src/witnesscalc_${circuit}.cpp

file="src/CMakeLists.txt"  # Replace this with the actual file name

# Define the string to be appended with circuit and HEADER variables
cmake_string=$(cat <<EOF

# ${circuit}
set(${header}_SOURCES \${LIB_SOURCES}
    ${circuit}.cpp
    witnesscalc_${circuit}.h
    witnesscalc_${circuit}.cpp
)

add_library(witnesscalc_${circuit} SHARED \${${header}_SOURCES})
add_library(witnesscalc_${circuit}Static STATIC \${${header}_SOURCES})
set_target_properties(witnesscalc_${circuit}Static PROPERTIES OUTPUT_NAME witnesscalc_${circuit})

add_executable(${circuit} main.cpp)
target_link_libraries(${circuit} witnesscalc_${circuit})

target_compile_definitions(witnesscalc_${circuit} PUBLIC CIRCUIT_NAME=${circuit})
target_compile_definitions(witnesscalc_${circuit}Static PUBLIC CIRCUIT_NAME=${circuit})
target_compile_definitions(${circuit} PUBLIC CIRCUIT_NAME=${circuit})
EOF
)

echo "$cmake_string" >> "$file"
