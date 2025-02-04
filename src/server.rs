use aws_nitro_enclaves_nsm_api::api::{ErrorCode, Request, Response};
use aws_nitro_enclaves_nsm_api::driver::{nsm_exit, nsm_init, nsm_process_request};
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::{types::ErrorObjectOwned, ResponsePayload};
use ring::agreement::{agree_ephemeral, EphemeralPrivateKey, UnparsedPublicKey, ECDH_P256};
use ring::rand::SystemRandom;
use serde_bytes::ByteBuf;
use std::sync::{Arc, Mutex};

use crate::generator::ProofType;
use crate::store::Store;
use crate::types::ProofRequest;
use crate::utils;
use crate::{generator::file_generator::FileGenerator, types::HelloResponse};

#[rpc(server, namespace = "openpassport")]
pub trait Rpc {
    #[method(name = "hello")]
    async fn hello(&self, user_pubkey: Vec<u8>) -> ResponsePayload<'static, HelloResponse>;
    #[method(name = "submit_request")]
    async fn submit_request(
        &self,
        uuid: String,
        nonce: Vec<u8>,
        cipher_text: Vec<u8>,
        auth_tag: Vec<u8>,
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
    store: Arc<Mutex<S>>,
    file_generator_sender: tokio::sync::mpsc::Sender<FileGenerator>,
}

impl<S> RpcServerImpl<S> {
    pub fn new(store: S, file_generator_sender: tokio::sync::mpsc::Sender<FileGenerator>) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
            file_generator_sender,
        }
    }
}

#[async_trait]
impl<S: Store + Sync + Send + 'static> RpcServer for RpcServerImpl<S> {
    async fn hello(&self, user_pubkey: Vec<u8>) -> ResponsePayload<'static, HelloResponse> {
        if user_pubkey.len() != 65 {
            return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                404, //BAD_REQUEST
                "Public key must be 65 bytes",
                None,
            ));
        };

        let rng = SystemRandom::new();

        let my_private_key = match EphemeralPrivateKey::generate(&ECDH_P256, &rng) {
            Ok(key) => key,
            Err(_) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    500, //INTERNAL_SERVER_ERROR
                    "Failed to generate ephemeral key",
                    None,
                ));
            }
        };

        let my_public_key = match my_private_key.compute_public_key() {
            Ok(pubkey) => pubkey,
            Err(_) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    500, //INTERNAL_SERVER_ERROR
                    "Failed to generate ephemeral key",
                    None,
                ));
            }
        }
        .as_ref()
        .to_vec();

        let their_public_key = UnparsedPublicKey::new(&ECDH_P256, user_pubkey.as_slice());

        let derived_key_result =
            match agree_ephemeral(my_private_key, &their_public_key, |shared_secret| {
                shared_secret.to_vec()
            }) {
                Ok(shared_secret) => shared_secret,
                Err(_) => {
                    return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                        500, //INTERNAL_SERVER_ERROR
                        "Failed to generate ephemeral key",
                        None,
                    ));
                }
            };

        let uuid_ = uuid::Uuid::new_v4();

        let mut store = match self.store.lock() {
            Ok(store) => store,
            Err(_) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    500, // INTERNAL_SERVER_ERROR
                    "Failed to store ephemeral key",
                    None,
                ));
            }
        };

        match store.insert_new_agreement(uuid_, derived_key_result) {
            Ok(_) => (),
            Err(_) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    500, //INTERNAL_SERVER_ERROR
                    "Failed to store ephemeral key",
                    None,
                ));
            }
        }

        drop(store);

        ResponsePayload::success(HelloResponse::new(uuid_, my_public_key).into())
    }

    //TODO: check if circuit exists
    async fn submit_request(
        &self,
        uuid: String,
        nonce: Vec<u8>,
        cipehr_text: Vec<u8>,
        auth_tag: Vec<u8>,
    ) -> ResponsePayload<'static, String> {
        let nonce = nonce.as_slice();
        let auth_tag = auth_tag.as_slice();
        let key = {
            let store = match self.store.lock() {
                Ok(store) => store,
                Err(_) => {
                    return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                        500, // INTERNAL_SERVER_ERROR
                        "Failed to store ephemeral key",
                        None,
                    ));
                }
            };

            let key = match store.get_shared_secret(&uuid) {
                Some(shared_secret) => shared_secret,
                None => {
                    return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                        404, // NOT_FOUND
                        "UUID not found",
                        None,
                    ));
                }
            };
            key
        };

        let key: [u8; 32] = key.try_into().unwrap();

        let decrypted_text = match utils::decrypt(key, cipehr_text, auth_tag, nonce) {
            Ok(text) => text,
            Err(e) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    404,
                    e,
                    Some("Failed to decrypt text".to_string()),
                ));
            }
        };

        let proof_request_type: ProofRequest = match serde_json::from_str(&decrypted_text) {
            Ok(proof_request_type) => proof_request_type,
            Err(_) => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    404,
                    "Failed to parse proof request", //make the error vague so that onlookers don't know what went wrong
                    None,
                ));
            }
        };

        let circuits = match proof_request_type {
            ProofRequest::Register { prove } => {
                let mut circuits = Vec::new();
                circuits.push(prove);
                // circuits.push(dsc);
                circuits
            }
            ProofRequest::Disclose { disclose } => {
                let mut circuits = Vec::new();
                circuits.push(disclose);
                circuits
            }
        };

        let circuits_length = circuits.len();

        for (i, circuit) in circuits.into_iter().enumerate() {
            let proof_type = if circuits_length == 2 {
                if i == 0 {
                    ProofType::Prove
                } else {
                    ProofType::Dsc
                }
            } else {
                ProofType::Disclose
            };

            let file_generator = FileGenerator::new(uuid.clone(), proof_type, circuit);

            match self.file_generator_sender.send(file_generator).await {
                Ok(()) => (),
                Err(e) => {
                    return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                        500, //INTERNAL_SERVER_ERROR
                        e.to_string(),
                        None,
                    ));
                }
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

        let fd = nsm_init();

        let result = match nsm_process_request(fd, request) {
            Response::Attestation { document } => ResponsePayload::success(document),
            Response::Error(err) => ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                500, //INTERNAL_SERVER_ERROR
                format!("{:?}", err),
                None,
            )),
            _ => {
                return ResponsePayload::error(ErrorObjectOwned::owned::<String>(
                    500, //INTERNAL_SERVER_ERROR
                    format!("{:?}", ErrorCode::InvalidResponse),
                    None,
                ));
            }
        };

        nsm_exit(fd);

        return result;
    }
}
