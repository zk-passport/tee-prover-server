FROM rust:1.81 as builder

RUN apt-get update
RUN apt-get install build-essential cmake libgmp-dev libsodium-dev nasm curl m4 -y

RUN git clone https://github.com/iden3/circom.git
RUN cd circom
RUN cd circom && cargo build --release
RUN cd circom && cargo install --path circom

WORKDIR /circuits
COPY . .

RUN ./build_gmp.sh host
# RUN ./pre_build_witnesses.sh /circuits/openpassport /circuits
# RUN ls src
RUN ./build_witnesses.sh /circuits
RUN ./build_witnesscalc_registerSha1Sha256Sha256Rsa655374096/src/registerSha1Sha256Sha256Rsa655374096 input.json o.wtns