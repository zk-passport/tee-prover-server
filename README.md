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
docker build -f Dockerfile.tee -t tee-server .
```
