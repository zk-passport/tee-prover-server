FROM witnesscalc-op:0.1

RUN apt-get update
RUN apt-get install build-essential cmake libgmp-dev libsodium-dev nasm curl m4 -y

WORKDIR /
RUN USER=root cargo new --bin tee-server
WORKDIR /tee-server
COPY ./rapidsnark ./rapidsnark

COPY ./zkeys ./zkeys 

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN cargo install --path .

CMD ["tee-server"]
