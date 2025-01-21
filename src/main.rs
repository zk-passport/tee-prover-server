mod generator;
mod store;
mod types;
mod utils;

use std::collections::HashMap;
use std::path;
use std::sync::{Arc, Mutex};

use generator::{
    file_generator::FileGenerator, proof_generator::ProofGenerator,
    witness_generator::WitnessGenerator, ProofType,
};
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::Server;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::ResponsePayload;
use ring::agreement::{agree_ephemeral, EphemeralPrivateKey, UnparsedPublicKey, ECDH_P256};
use ring::rand::SystemRandom;
use store::{HashMapStore, Store};
use types::{HelloResponse, ProofRequest};

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
}

#[tokio::main]
async fn main() {
    let server = Server::builder().build("0.0.0.0:3001").await.unwrap();

    let circuit_folder = "../circuits";
    let circuit_file_prefix = "build_witnesscalc_";

    let zkey_folder = "./zkeys";
    let mut circuit_zkey_map = HashMap::new();
    circuit_zkey_map.insert(
        "proveSha1Sha1Sha1Rsa655374096",
        "proveSha1Sha1Sha1Rsa655374096.zkey",
    );
    circuit_zkey_map.insert(
        "proveSha256Sha256Sha256EcdsaBrainpoolP256r1",
        "proveSha256Sha256Sha256EcdsaBrainpoolP256r1.zkey",
    );

    let rapid_snark_path_exe = path::Path::new("./rapidsnark")
        .join("package")
        .join("bin")
        .join("prover");

    if !rapid_snark_path_exe.exists() {
        panic!("rapid snark path does not exist!");
    }

    for (key, value) in circuit_zkey_map.iter() {
        //check if these zkey paths exist
        let zkey_path = path::Path::new(zkey_folder).join(value);
        if !zkey_path.exists() {
            let zkey_path_str = zkey_path.to_str().unwrap();
            panic!("zkey {zkey_path_str} does not exist!");
        }

        let circuit_path = path::Path::new(circuit_folder)
            .join(format!("{}{}", circuit_file_prefix, key))
            .join("src")
            .join(key);
        if !circuit_path.exists() {
            let circuit_path_str = circuit_path.to_str().unwrap();
            panic!("circuit {circuit_path_str} does not exist!");
        }
    }

    let rapid_snark_path = rapid_snark_path_exe.into_os_string().into_string().unwrap();

    let (file_generator_sender, mut file_generator_receiver) = tokio::sync::mpsc::channel(10);
    let (witness_generator_sender, mut witness_generator_receiver) = tokio::sync::mpsc::channel(10);
    let (proof_generator_sender, mut proof_generator_receiver) = tokio::sync::mpsc::channel(10);

    let addr = server.local_addr().unwrap();
    let handle =
        server.start(RpcServerImpl::new(HashMapStore::new(), file_generator_sender).into_rpc());

    let url = format!("http://{}", addr);

    println!("Server running in url: {}", url);

    let circuit_zkey_map_arc = Arc::new(circuit_zkey_map);

    tokio::select! {
        _ = handle.stopped() => {
            //delete tmp folders
            println!("Server stopped");
        }

        _ = async {
            while let Some(file_generator) = file_generator_receiver.recv().await {
                let witness_generator_clone = witness_generator_sender.clone();
                tokio::spawn(async move {
                    let (uuid, proof_type, circuit_name) = match file_generator.run().await {
                        Ok((uuid, proof_type, circuit_name)) => (uuid, proof_type, circuit_name),
                        Err(e) => {
                            dbg!(e);
                            //when adding db logic add failure to status
                            return;
                        }
                    };
                    let _ = witness_generator_clone.send(WitnessGenerator::new(
                        uuid,
                        proof_type,
                        circuit_name
                    )).await;
                });
            }
        } => {}

        _ = async {
            while let Some(witness_generator) = witness_generator_receiver.recv().await {
                let circuit_zkey_map_arc_clone = Arc::clone(&circuit_zkey_map_arc);
                let proof_generator_sender_clone = proof_generator_sender.clone();
                tokio::spawn(async move {
                    //handle error as well
                    if let Ok((uuid, proof_type, circuit_name)) = witness_generator
                        .run(circuit_folder, circuit_file_prefix)
                        .await {
                            let zkey_file = circuit_zkey_map_arc_clone.get(circuit_name.as_str()).unwrap();
                            let zkey_file_path = path::Path::new(zkey_folder).join(zkey_file).to_str().unwrap().to_string();
                            match proof_generator_sender_clone.send(ProofGenerator::new(
                                uuid,
                                proof_type,
                                zkey_file_path,
                            )).await {
                                Ok(_) => {},
                                Err(e) => {
                                    dbg!(e);
                                    //when adding db logic add failure to status
                                    return;
                                }
                            }
                        }
                });
            }
        } => {}

        _ = async {
            while let Some(proof_generator) = proof_generator_receiver.recv().await {
                let _ = proof_generator.run(&rapid_snark_path).await;
            }
        } => {}
    }
}
