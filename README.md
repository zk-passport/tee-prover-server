# Prover server

The prover server allows a seamless interface to request proofs from a server. It also allows you to encrypt your requests to the server by making use of the NSM attestation API.

## Build

1. To build the Dockerfile first run:

```sh
git submodule update --init
git submodule update --init
cd ../rapidsnark
git submodule update --init
cd ..
```

2. Assuming you have yarn installed, install node_modules in the `openpassport/circuits` directory:

```sh
cd openpassport/circuits
yarn
cd ../../
```

3. Building the docker image:

```sh
docker build --build-arg PROOFTYPE=<PROOFTYPE> -f Dockerfile.tee -t <IMAGE_NAME> .
```

Where the `<PROOFTYPE>` can be one of: {register, dsc, disclose}.

## Running the server

The following options can be used to run the server:

```sh
Options:
  -s, --server-address <SERVER_ADDRESS>
          Web server bind address (e.g., 0.0.0.0:3001) [default: 0.0.0.0:3001]
  -d, --database-url <DATABASE_URL>
          PostgreSQL database connection URL [default: postgres://postgres:mysecretpassword@localhost:5433/db]
  -c, --circuit-folder <CIRCUIT_FOLDER>
          Circuit folder path [default: ../circuits]
  -k, --zkey-folder <ZKEY_FOLDER>
          ZKey folder path [default: ./zkeys]
  -z, --circuit-zkey-map <CIRCUIT_ZKEY_MAP>
          Witness calc circuit to zkey mapper
  -r, --rapidsnark-path <RAPIDSNARK_PATH>
          Rapidsnark path [default: ./rapidsnark]
  -h, --help
          Print help
```

When running the server in a nitro enclave please make sure you have two proxies running on the ec2 instance. One would be for the RPC server and the other for the DB.

```sh
# Install socat
sudo dnf install socat -y
socat tcp-listen:8888,fork,reuseaddr vsock-connect:<ENCLAVE_ID>:8888 # for the rpc server
socat vsock-listen:8889,fork,reuseaddr TCP4:<DB_HOST>:<DB_PORT> # for the db
```

# API

This API follows the JSON-RPC 2.0 protocol and operates under the `openpassport` namespace.

### 1. `hello`

**Description:**
The first part of an ECDH handshake. The user sends their public key along with a UUID that is linked to their session ID when scanning the QR code.

**Method Name:** `openpassport_hello`

**Request Parameters:**

- `user_pubkey` (Vec<u8>): The public key of the user.
- `uuid` (String): A unique identifier for the request.

**Response:**
Returns a `ResponsePayload` containing `HelloResponse` which is the request UUID and the attestation response. Please verify the response before making the second request.

---

### 2. `submit_request`

**Description:**
Submits an encrypted request along with authentication data. The encryption scheme used is AES-GCM.

**Method Name:** `openpassport_submit_request`

**Request Parameters:**

- `uuid` (String): A unique identifier for the request.
- `nonce` (Vec<u8>): A cryptographic nonce.
- `cipher_text` (Vec<u8>): The encrypted request payload.
- `auth_tag` (Vec<u8>): The authentication tag for integrity verification.

**Response:**
Returns a `ResponsePayload` containing the UUID.

---

### 3. `attestation`

**Description:**
Requests attestation for user data and cryptographic parameters.

**Method Name:** `openpassport_attestation`

**Request Parameters:**

- `user_data` (Option<Vec<u8>>): Optional user-related data.
- `nonce` (Option<Vec<u8>>): Optional cryptographic nonce.
- `public_key` (Option<Vec<u8>>): Optional public key.

**Response:**
Returns a `ResponsePayload` containing attestation data as a vector of bytes.

## Usage

Clients can send JSON-RPC requests to the OpenPassport API endpoint, following the standard JSON-RPC 2.0 format:

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "openpassport_hello",
  "params": {
    "user_pubkey": "...",
    "uuid": "550e8400-e29b-41d4-a716-446655440000"
  },
  "id": 1
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "uuid": "550e8400-e29b-41d4-a716-446655440000",
    "attestation": [...]
  }
}
```
