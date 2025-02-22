FROM public.ecr.aws/docker/library/rust:1.81-bookworm as rust-builder

RUN apt-get update
RUN apt-get install build-essential cmake libgmp-dev libsodium-dev nasm curl m4 -y

RUN git clone https://github.com/iden3/circom.git
RUN cd circom
RUN cd circom && cargo build --release
RUN cd circom && cargo install --path circom

WORKDIR /self
COPY ./self .

WORKDIR /circuits
COPY ./witnesscalc .
RUN ./build_gmp.sh host
RUN ./build_witnesses.sh /circuits

WORKDIR /rapidsnark
COPY ./rapidsnark .
RUN ./build_gmp.sh host && \
    mkdir build_prover && cd build_prover && \
    cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=../package && \
    make -j16 && make install

WORKDIR /
RUN USER=root cargo new --bin tee-server
WORKDIR /tee-server

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

ARG PROOFTYPE=${PROOFTYPE}
RUN cargo build --release --features $PROOFTYPE
RUN rm src/*.rs

COPY ./src ./src

RUN cargo install --path .

COPY ./zkeys ./zkeys 

ENTRYPOINT ["tee-server"]
