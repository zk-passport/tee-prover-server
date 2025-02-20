######## Enclave image ########

FROM public.ecr.aws/docker/library/rust:1.81-bookworm AS rust-builder
WORKDIR /src/
COPY Cargo.toml Cargo.lock ./
COPY src src/

ARG PROOFTYPE=${PROOFTYPE}
RUN cargo build --locked --release --features $PROOFTYPE

FROM public.ecr.aws/docker/library/debian:12.6-slim@sha256:2ccc7e39b0a6f504d252f807da1fc4b5bcd838e83e4dec3e2f57b2a4a64e7214 AS nitro-enclave

RUN apt-get update
RUN apt-get install build-essential cmake libgmp-dev libsodium-dev nasm curl m4 netcat-traditional socat iproute2 git jq unzip libc6 -y

WORKDIR /rapidsnark
COPY ./rapidsnark .
RUN ./build_gmp.sh host && \
    mkdir build_prover && cd build_prover && \
    cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=../package && \
    make -j16 && make install

COPY start.sh /usr/local/bin
RUN chown root:root /usr/local/bin/start.sh
RUN chmod 755 /usr/local/bin/start.sh
ARG PROOFTYPE=${PROOFTYPE}

COPY --from=rust-builder /src/target/release/tee-server /usr/local/bin/
WORKDIR /
COPY circuits /circuits

COPY ./zkeys/$PROOFTYPE /zkeys

CMD ["/usr/local/bin/start.sh"]