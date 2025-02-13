use aws_nitro_enclaves_nsm_api::api::{ErrorCode, Request, Response};
use aws_nitro_enclaves_nsm_api::driver::nsm_process_request;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types;
use jsonrpsee::{types::ErrorObjectOwned, ResponsePayload};
use p256::ecdh::EphemeralSecret;
use p256::elliptic_curve::PublicKey;
use rand_core::{CryptoRng, OsRng, RngCore};
use serde_bytes::ByteBuf;
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

use crate::store::Store;
use crate::types::ProofRequest;
use crate::utils::{self, nsm_get_random};
use crate::{generator::file_generator::FileGenerator, types::HelloResponse};

#[rpc(server, namespace = "openpassport")]
pub trait Rpc {
    #[method(name = "hello")]
    async fn hello(
        &self,
        user_pubkey: Vec<u8>,
        uuid: uuid::Uuid,
    ) -> ResponsePayload<'static, HelloResponse>;
    #[method(name = "submit_request")]
    async fn submit_request(
        &self,
        uuid: String,
        nonce: Vec<u8>,
        cipher_text: Vec<u8>,
        auth_tag: Vec<u8>,
        onchain: bool,
    ) -> ResponsePayload<'static, String>;
    #[method(name = "attestation")]
    async fn attestation(
        &self,
        user_data: Option<Vec<u8>>,
        nonce: Option<Vec<u8>>,
        public_key: Option<Vec<u8>>,
    ) -> ResponsePayload<'static, Vec<u8>>;
}

pub struct RpcServerImpl<S> {
    fd: i32,
    store: Arc<Mutex<S>>,
    file_generator_sender: tokio::sync::mpsc::Sender<(bool, FileGenerator)>,
    circuit_zkey_map: Arc<HashMap<String, String>>,
}

impl<S> RpcServerImpl<S> {
    pub fn new(
        fd: i32,
        store: S,
        file_generator_sender: tokio::sync::mpsc::Sender<(bool, FileGenerator)>,
        circuit_zkey_map: Arc<HashMap<String, String>>,
    ) -> Self {
        Self {
            fd,
            store: Arc::new(Mutex::new(store)),
            file_generator_sender,
            circuit_zkey_map,
        }
    }
}

#[async_trait]
impl<S: Store + Sync + Send + 'static> RpcServer for RpcServerImpl<S> {
    async fn hello(
        &self,
        user_pubkey: Vec<u8>,
        uuid: uuid::Uuid,
    ) -> ResponsePayload<'static, HelloResponse> {
        if user_pubkey.len() != 65 {
            return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                types::ErrorCode::InvalidRequest.code(), //BAD_REQUEST
                "Public key must be 65 bytes",
                None,
            ));
        };

        // let mut nitro_rng = NitroRng::new(self.fd);
        let mut nitro_rng = OsRng;

        let my_private_key = EphemeralSecret::random(&mut nitro_rng);
        let my_public_key = PublicKey::from(&my_private_key).to_sec1_bytes().to_vec();

        let attestation = match utils::get_attestation(
            self.fd,
            Some(user_pubkey.clone()),
            None,
            Some(my_public_key.clone()),
        ) {
            Ok(attestation) => attestation,
            Err(err) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    types::ErrorCode::InternalError.code(), //INTERNAL_SERVER_ERROR
                    format!("{:?}", err),
                    None,
                ));
            }
        };

        let their_public_key = match PublicKey::from_sec1_bytes(&user_pubkey) {
            Ok(pubkey) => pubkey,
            Err(err) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    types::ErrorCode::InvalidParams.code(), //INVALID_PARAMS
                    format!("{:?}", err),
                    None,
                ));
            }
        };

        let derived_key_result = my_private_key
            .diffie_hellman(&their_public_key)
            .raw_secret_bytes()
            .to_vec();

        let mut store = match self.store.lock() {
            Ok(store) => store,
            Err(_) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    types::ErrorCode::InternalError.code(), // INTERNAL_SERVER_ERROR
                    "Failed to store ephemeral key",
                    None,
                ));
            }
        };

        match store.insert_new_agreement(uuid, derived_key_result) {
            Ok(_) => (),
            Err(_) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    types::ErrorCode::InvalidRequest.code(), //INTERNAL_SERVER_ERROR
                    "UUID already exists",
                    None,
                ));
            }
        }

        drop(store);

        ResponsePayload::success(HelloResponse::new(uuid, my_public_key).into())
    }

    //TODO: check if circuit exists
    async fn submit_request(
        &self,
        uuid: String,
        nonce: Vec<u8>,
        cipher_text: Vec<u8>,
        auth_tag: Vec<u8>,
        onchain: bool,
    ) -> ResponsePayload<'static, String> {
        let nonce = nonce.as_slice();
        let auth_tag = auth_tag.as_slice();
        let key = {
            let store = match self.store.lock() {
                Ok(store) => store,
                Err(_) => {
                    return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                        types::ErrorCode::InternalError.code(), //INTERNAL_SERVER_ERROR
                        "Failed to store ephemeral key",
                        None,
                    ));
                }
            };

            let key = match store.get_shared_secret(&uuid) {
                Some(shared_secret) => shared_secret,
                None => {
                    return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                        types::ErrorCode::InvalidRequest.code(),
                        "UUID not found",
                        None,
                    ));
                }
            };
            key
        };

        let key: [u8; 32] = match key.try_into() {
            Ok(key) => key,
            Err(_) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    types::ErrorCode::InternalError.code(), //INTERNAL_SERVER_ERROR
                    "Failed to store ephemeral key",
                    None,
                ));
            }
        };

        let decrypted_text: String = match utils::decrypt(key, cipher_text, auth_tag, nonce) {
            Ok(text) => text,
            Err(_) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    types::ErrorCode::InvalidRequest.code(),
                    "Failed to decrypt text",
                    None,
                ));
            }
        };

        let proof_request_type = match serde_json::from_str::<ProofRequest>(&decrypted_text) {
            Ok(proof_request_type) => {
                let circuit_name = proof_request_type.circuit().name.clone();
                if !self.circuit_zkey_map.contains_key(&circuit_name) {
                    return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                        types::ErrorCode::InvalidRequest.code(),
                        format!("Could not find the given circuit name: {}", &circuit_name),
                        None,
                    ));
                }
                proof_request_type
            }
            Err(_) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    types::ErrorCode::InvalidRequest.code(),
                    "Failed to parse proof request",
                    None,
                ));
            }
        };

        let mut allowed_proof_type = "";
        if cfg!(feature = "register") {
            allowed_proof_type = "register";
        } else if cfg!(feature = "dsc") {
            allowed_proof_type = "dsc";
        } else {
            allowed_proof_type = "disclose";
        }

        let invalid_proof_type_response =
            ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                types::ErrorCode::InvalidRequest.code(), //BAD REQUEST
                format!("This endpoint only allows {} inputs", allowed_proof_type),
                None,
            ));

        match proof_request_type {
            ProofRequest::Register { .. } => {
                if !cfg!(feature = "register") {
                    return invalid_proof_type_response;
                }
            }
            ProofRequest::Dsc { .. } => {
                if !cfg!(feature = "dsc") {
                    return invalid_proof_type_response;
                }
            }
            ProofRequest::Disclose { .. } => {
                if !cfg!(feature = "disclose") {
                    return invalid_proof_type_response;
                }
            }
        };

        let file_generator = FileGenerator::new(uuid.clone(), proof_request_type);
        match self
            .file_generator_sender
            .send((onchain, file_generator))
            .await
        {
            Ok(()) => (),
            Err(e) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    types::ErrorCode::InternalError.code(), //INTERNAL_SERVER_ERROR
                    e.to_string(),
                    None,
                ));
            }
        }

        ResponsePayload::success(uuid)
    }

    async fn attestation(
        &self,
        user_data: Option<Vec<u8>>,
        nonce: Option<Vec<u8>>,
        public_key: Option<Vec<u8>>,
    ) -> ResponsePayload<'static, Vec<u8>> {
        let request = Request::Attestation {
            user_data: user_data.map(|buf| ByteBuf::from(buf)),
            nonce: nonce.map(|buf| ByteBuf::from(buf)),
            public_key: public_key.map(|buf| ByteBuf::from(buf)),
        };

        let result = match nsm_process_request(self.fd, request) {
            Response::Attestation { document } => ResponsePayload::success(document),
            Response::Error(err) => ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                types::ErrorCode::InternalError.code(), //INTERNAL_SERVER_ERROR
                format!("{:?}", err),
                None,
            )),
            _ => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    types::ErrorCode::InternalError.code(), //INTERNAL_SERVER_ERROR
                    format!("{:?}", ErrorCode::InvalidResponse),
                    None,
                ));
            }
        };

        return result;
    }
}

pub struct NitroRng {
    fd: i32, // File descriptor for NitroSecureModule
}

impl NitroRng {
    pub fn new(fd: i32) -> Self {
        Self { fd }
    }
}

impl RngCore for NitroRng {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        unsafe {
            let mut buf_len = dest.len();
            let res = nsm_get_random(self.fd, dest.as_mut_ptr(), &mut buf_len);
            match res {
                ErrorCode::Success => (),
                _ => panic!("Failed to get random bytes: {:?}", res),
            }
        }
    }

    fn next_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.fill_bytes(&mut buf);
        u32::from_le_bytes(buf)
    }

    fn next_u64(&mut self) -> u64 {
        let mut buf = [0u8; 8];
        self.fill_bytes(&mut buf);
        u64::from_le_bytes(buf)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        unsafe {
            let mut buf_len = dest.len();
            let res = nsm_get_random(self.fd, dest.as_mut_ptr(), &mut buf_len);
            match res {
                ErrorCode::Success => Ok(()),
                _ => Err(rand_core::Error::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Could not generate random data",
                ))),
            }
        }
    }
}

impl CryptoRng for NitroRng {}
