include_directories(
    ../src
    ../build
    ../depends/json/single_include)

link_libraries(${GMP_LIB})

add_definitions(${GMP_DEFINIONS})

if(USE_ASM)
    if(ARCH MATCHES "arm64")
            add_definitions(-DUSE_ASM -DARCH_ARM64)
    elseif(ARCH MATCHES "x86_64")
            add_definitions(-DUSE_ASM -DARCH_X86_64)
    endif()
endif()

if(USE_ASM AND ARCH MATCHES "x86_64")

    if (CMAKE_HOST_SYSTEM_NAME MATCHES "Darwin" AND NOT TARGET_PLATFORM MATCHES "^android(_x86_64)?")
        set(NASM_FLAGS -fmacho64 --prefix _)
    else()
        set(NASM_FLAGS -felf64 -DPIC)
    endif()

    add_custom_command(OUTPUT ${CMAKE_SOURCE_DIR}/build/fq_asm.o
        COMMAND nasm ${NASM_FLAGS} fq.asm -o fq_asm.o
        DEPENDS ${CMAKE_SOURCE_DIR}/build/fq.asm
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/build)

    add_custom_command(OUTPUT ${CMAKE_SOURCE_DIR}/build/fr_asm.o
        COMMAND nasm ${NASM_FLAGS} fr.asm -o fr_asm.o
        DEPENDS ${CMAKE_SOURCE_DIR}/build/fr.asm
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/build)
endif()

set(FR_SOURCES
    ../build/fr.hpp
    ../build/fr.cpp
    )

if(USE_ASM)
    if(ARCH MATCHES "arm64")
        set(FR_SOURCES ${FR_SOURCES} ../build/fr_raw_arm64.s ../build/fr_raw_generic.cpp ../build/fr_generic.cpp)
    elseif(ARCH MATCHES "x86_64")
        set(FR_SOURCES ${FR_SOURCES} ../build/fr_asm.o)
    endif()
else()
    set(FR_SOURCES ${FR_SOURCES} ../build/fr_generic.cpp ../build/fr_raw_generic.cpp)
endif()

add_library(fr STATIC ${FR_SOURCES})
set_target_properties(fr PROPERTIES POSITION_INDEPENDENT_CODE ON)

link_libraries(fr)


add_executable(tests tests.cpp)
add_executable(test_platform test_platform.cpp)


set(LIB_SOURCES
    calcwit.cpp
    witnesscalc.h
    witnesscalc.cpp
    )


# registerSha1Sha256Sha256Rsa655374096
set(REGISTERSHA1SHA256SHA256RSA655374096_SOURCES ${LIB_SOURCES}
    registerSha1Sha256Sha256Rsa655374096.cpp
    witnesscalc_registerSha1Sha256Sha256Rsa655374096.h
    witnesscalc_registerSha1Sha256Sha256Rsa655374096.cpp
)

add_library(witnesscalc_registerSha1Sha256Sha256Rsa655374096 SHARED ${REGISTERSHA1SHA256SHA256RSA655374096_SOURCES})
add_library(witnesscalc_registerSha1Sha256Sha256Rsa655374096Static STATIC ${REGISTERSHA1SHA256SHA256RSA655374096_SOURCES})
set_target_properties(witnesscalc_registerSha1Sha256Sha256Rsa655374096Static PROPERTIES OUTPUT_NAME witnesscalc_registerSha1Sha256Sha256Rsa655374096)

add_executable(registerSha1Sha256Sha256Rsa655374096 main.cpp)
target_link_libraries(registerSha1Sha256Sha256Rsa655374096 witnesscalc_registerSha1Sha256Sha256Rsa655374096)

target_compile_definitions(witnesscalc_registerSha1Sha256Sha256Rsa655374096 PUBLIC CIRCUIT_NAME=registerSha1Sha256Sha256Rsa655374096)
target_compile_definitions(witnesscalc_registerSha1Sha256Sha256Rsa655374096Static PUBLIC CIRCUIT_NAME=registerSha1Sha256Sha256Rsa655374096)
target_compile_definitions(registerSha1Sha256Sha256Rsa655374096 PUBLIC CIRCUIT_NAME=registerSha1Sha256Sha256Rsa655374096)

# registerSha256Sha256Sha256EcdsaBrainpoolP256r1
set(REGISTERSHA256SHA256SHA256ECDSABRAINPOOLP256R1_SOURCES ${LIB_SOURCES}
    registerSha256Sha256Sha256EcdsaBrainpoolP256r1.cpp
    witnesscalc_registerSha256Sha256Sha256EcdsaBrainpoolP256r1.h
    witnesscalc_registerSha256Sha256Sha256EcdsaBrainpoolP256r1.cpp
)

add_library(witnesscalc_registerSha256Sha256Sha256EcdsaBrainpoolP256r1 SHARED ${REGISTERSHA256SHA256SHA256ECDSABRAINPOOLP256R1_SOURCES})
add_library(witnesscalc_registerSha256Sha256Sha256EcdsaBrainpoolP256r1Static STATIC ${REGISTERSHA256SHA256SHA256ECDSABRAINPOOLP256R1_SOURCES})
set_target_properties(witnesscalc_registerSha256Sha256Sha256EcdsaBrainpoolP256r1Static PROPERTIES OUTPUT_NAME witnesscalc_registerSha256Sha256Sha256EcdsaBrainpoolP256r1)

add_executable(registerSha256Sha256Sha256EcdsaBrainpoolP256r1 main.cpp)
target_link_libraries(registerSha256Sha256Sha256EcdsaBrainpoolP256r1 witnesscalc_registerSha256Sha256Sha256EcdsaBrainpoolP256r1)

target_compile_definitions(witnesscalc_registerSha256Sha256Sha256EcdsaBrainpoolP256r1 PUBLIC CIRCUIT_NAME=registerSha256Sha256Sha256EcdsaBrainpoolP256r1)
target_compile_definitions(witnesscalc_registerSha256Sha256Sha256EcdsaBrainpoolP256r1Static PUBLIC CIRCUIT_NAME=registerSha256Sha256Sha256EcdsaBrainpoolP256r1)
target_compile_definitions(registerSha256Sha256Sha256EcdsaBrainpoolP256r1 PUBLIC CIRCUIT_NAME=registerSha256Sha256Sha256EcdsaBrainpoolP256r1)
