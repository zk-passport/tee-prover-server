mod args;
mod db;
mod generator;
mod server;
mod store;
mod types;
mod utils;

use std::collections::HashMap;
use std::path;
use std::sync::Arc;

use clap::Parser;
use db::{
    create_proof_status, listen_status_update, update_proof, update_proof_status,
    StatusUpdatePayload,
};
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use generator::{proof_generator::ProofGenerator, witness_generator::WitnessGenerator};
use jsonrpsee::server::Server;
use server::RpcServer;
use sqlx::postgres::PgPoolOptions;
use store::HashMapStore;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
use types::ConnectionMap;

pub async fn handle_connection(
    ws_stream: WebSocketStream<TcpStream>,
    connection_map: ConnectionMap,
) {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let (update_tx, mut update_rx) = tokio::sync::mpsc::channel::<StatusUpdatePayload>(100);

    match ws_receiver.next().await {
        Some(Ok(message)) => {
            connection_map
                .write()
                .await
                .insert(message.to_string(), update_tx);
        }
        Some(Err(e)) => {
            println!("Error receiving message: {}", e);
            return;
        }
        None => {
            println!("Hi");
            return;
        }
    };

    let send_task = tokio::spawn(async move {
        while let Some(payload) = update_rx.recv().await {
            if let Err(e) = ws_sender
                .send(Message::text(&serde_json::to_string(&payload).unwrap()))
                .await
            {
                println!("Error sending message: {}", e);
                break;
            }
        }
    });

    let receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            println!("Received message: {}", msg);
        }
    });

    tokio::select! {
        _ = send_task => {}
        _ = receive_task => {}
    };

    connection_map.write().await.retain(|_, v| !v.is_closed());
    println!("Connection closed");
}

#[tokio::main]
async fn main() {
    let config = args::Config::parse();
    let server_url = config.server_address;
    let ws_server_url = config.ws_server_url;

    let server = Server::builder().build(server_url).await.unwrap();

    let pool = match PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            panic!("Error: {:?}", e);
        }
    };

    let circuit_folder = config.circuit_folder;
    let circuit_file_prefix = config.circuit_file_prefix;
    let zkey_folder = config.zkey_folder;

    let mut circuit_zkey_map = HashMap::new();
    for pair in config.circuit_zkey_map.iter() {
        circuit_zkey_map.insert(pair.0.clone(), pair.1.clone());
    }

    for (key, value) in circuit_zkey_map.iter() {
        //check if these zkey paths exist
        let zkey_path = path::Path::new(&zkey_folder).join(value);
        if !zkey_path.exists() {
            let zkey_path_str = zkey_path.to_str().unwrap();
            panic!("zkey {zkey_path_str} does not exist!");
        }

        let circuit_path = path::Path::new(&circuit_folder)
            .join(format!("{}{}", circuit_file_prefix, key))
            .join("src")
            .join(key);
        if !circuit_path.exists() {
            let circuit_path_str = circuit_path.to_str().unwrap();
            panic!("circuit {circuit_path_str} does not exist!");
        }
    }

    let circuit_zkey_map_arc = Arc::new(circuit_zkey_map);

    let connection_map: ConnectionMap = Arc::new(RwLock::new(HashMap::new()));

    let rapid_snark_path_exe = path::Path::new("./rapidsnark")
        .join("package")
        .join("bin")
        .join("prover");

    if !rapid_snark_path_exe.exists() {
        panic!("rapid snark path does not exist!");
    }
    let rapid_snark_path = rapid_snark_path_exe.into_os_string().into_string().unwrap();

    let (file_generator_sender, mut file_generator_receiver) = tokio::sync::mpsc::channel(10);
    let (witness_generator_sender, mut witness_generator_receiver) = tokio::sync::mpsc::channel(10);
    let (proof_generator_sender, mut proof_generator_receiver) = tokio::sync::mpsc::channel(10);

    let server_addr = server.local_addr().unwrap();

    let handle = server
        .start(server::RpcServerImpl::new(HashMapStore::new(), file_generator_sender).into_rpc());

    println!("Server running on: http://{}", server_addr);

    let listener = TcpListener::bind(&ws_server_url)
        .await
        .expect("Failed to bind");

    tokio::select! {
        _ = handle.stopped() => {
            //delete tmp folders
            println!("Server stopped");
        }

        _ = async {
                let connection_map_clone = Arc::clone(&connection_map);
                loop {
                    match listener.accept().await {
                        Ok((stream, _)) => {
                            let peer_addr = stream
                                .peer_addr()
                                .expect("Connected streams should have a peer address");
                            println!("New connection: {}", peer_addr);

                            let ws_stream = accept_async(stream)
                                .await
                                .expect("Error during the websocket handshake");
                            tokio::spawn(handle_connection(ws_stream, Arc::clone(&connection_map_clone)));
                        },
                        Err(e) => {
                            println!("Error accepting connection: {}", e);
                        }
                    }

                }
        } => {}

        _ = async {
            let _ = listen_status_update(&pool, "status_update", Arc::clone(&connection_map)).await;
        } => {}

        _ = async {
            while let Some(file_generator) = file_generator_receiver.recv().await {
                let uuid = file_generator.uuid();
                let proof_type = file_generator.proof_type();

                //maybe it's better to panic / unwrap here?
                let _ = create_proof_status(uuid, &proof_type, db::types::Status::Pending, &pool).await;

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

                let circuit_folder = circuit_folder.clone();
                let circuit_file_prefix = circuit_file_prefix.clone();
                let zkey_folder = zkey_folder.clone();
                tokio::spawn(async move {
                    //handle error as well
                    if let Ok((uuid, proof_type, circuit_name)) = witness_generator
                        .run(&circuit_folder, &circuit_file_prefix)
                        .await {
                            let zkey_file = circuit_zkey_map_arc_clone.get(circuit_name.as_str()).unwrap();
                            let zkey_file_path = path::Path::new(&zkey_folder).join(zkey_file).to_str().unwrap().to_string();
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
                let uuid = proof_generator.uuid();
                let proof_type = proof_generator.proof_type();
                let _ = update_proof_status(uuid.clone(), &proof_type, db::types::Status::WitnessGenerated, &pool).await;
                let _ = proof_generator.run(&rapid_snark_path).await;
                let _ = update_proof(uuid, &proof_type, &pool).await;
            }
        } => {}
    }

    println!("hi");
}

//proveSha1Sha1Sha1Rsa655374096=proveSha1Sha1Sha1Rsa655374096.zkey
//proveSha256Sha256Sha256EcdsaBrainpoolP256r1=proveSha256Sha256Sha256EcdsaBrainpoolP256r1.zkey
