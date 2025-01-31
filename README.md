# Prover server

## Build

1. To build the Dockerfile first run:

```
git submodule update --init
cd witnesscalc
git submodule update --init
cd ../rapidsnark
git submodule update --init
cd ..
```

2. Assuming you have yarn installed, install node_modules in the `openpassport/circuits` directory:

```
cd openpassport/circuits
yarn
cd ../../
```

3. Building the docker image:

```
docker build -t tee-server .
```

4. Running the image

```
docker run \
    --add-host host.docker.internal:host-gateway \
    -p 127.0.0.1:3001:3001 \
    -p 127.0.0.1:3002:3002 \
    -it tee-server \
    -r ../rapidsnark \
    -z registerSha1Sha256Sha256Rsa655374096=registerSha1Sha256Sha256Rsa655374096.zkey \
    -z registerSha256Sha256Sha256EcdsaBrainpoolP256r1=registerSha256Sha256Sha256EcdsaBrainpoolP256r1.zkey \
    --database-url=postgres://postgres:mysecretpassword@host.docker.internal:5433/db
```
