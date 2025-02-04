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

use aws_nitro_enclaves_nsm_api::driver::{nsm_exit, nsm_init};
use clap::Parser;
use db::{create_proof_status, update_proof, update_proof_status};
use generator::{proof_generator::ProofGenerator, witness_generator::WitnessGenerator};
use jsonrpsee::server::Server;
use server::RpcServer;
use sqlx::postgres::PgPoolOptions;
use store::HashMapStore;

#[tokio::main]
async fn main() {
    let config = args::Config::parse();
    let server_url = config.server_address;

    let server = Server::builder().build(server_url).await.unwrap();

    let (file_generator_sender, mut file_generator_receiver) = tokio::sync::mpsc::channel(10);
    let (witness_generator_sender, mut witness_generator_receiver) = tokio::sync::mpsc::channel(10);
    let (proof_generator_sender, mut proof_generator_receiver) = tokio::sync::mpsc::channel(10);

    let server_addr = server.local_addr().unwrap();
    let fd = nsm_init();

    let handle = server.start(
        server::RpcServerImpl::new(fd, HashMapStore::new(), file_generator_sender).into_rpc(),
    );

    // handle.stopped().await

    println!("Server running on: http://{}", server_addr);

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

    let mut circuit_zkey_map: HashMap<String, String> = HashMap::new();
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

    let rapid_snark_path_exe = path::Path::new(&config.rapidsnark_path)
        .join("package")
        .join("bin")
        .join("prover");

    if !rapid_snark_path_exe.exists() {
        panic!("rapid snark path does not exist!");
    }
    let rapid_snark_path = rapid_snark_path_exe.into_os_string().into_string().unwrap();

    tokio::select! {
        _ = handle.stopped() => {
            //delete tmp folders
            println!("Server stopped");
            nsm_exit(fd);
        }

    _ = async {
        while let Some(file_generator) = file_generator_receiver.recv().await {
            let uuid = file_generator.uuid();
            let proof_type = file_generator.proof_type();

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
            //handle error
            let _res = proof_generator.run(&rapid_snark_path).await;
            let _ = update_proof(uuid, &proof_type, &pool).await;
        }
    } => {}
    }
}
