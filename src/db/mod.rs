use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;
use tokio::io;

use crate::{types::ProofType, utils::get_tmp_folder_path};
pub mod types;

pub async fn create_proof_status(
    uuid: &String,
    proof_type: &ProofType,
    db: &sqlx::Pool<sqlx::Postgres>,
) -> Result<(), sqlx::Error> {
    let proof_type_id: i32 = proof_type.into();
    let now = Utc::now();

    let status: i32 = types::Status::Pending.into();

    match sqlx::query(
        "INSERT INTO proofs (proof_type, request_id, status, created_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(proof_type_id)
    .bind(sqlx::types::Uuid::parse_str(&uuid).unwrap())
    .bind(status)
    .bind(now)
    .execute(db)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    }
}

pub async fn set_witness_generated(
    uuid: String,
    db: &sqlx::Pool<sqlx::Postgres>,
) -> Result<(), sqlx::Error> {
    let status: i32 = types::Status::WitnessGenerated.into();
    let now = Utc::now();

    match sqlx::query(&format!(
        "UPDATE proofs SET status = $1, witness_generated_at = $2 WHERE request_id = $3",
    ))
    .bind(status)
    .bind(now)
    .bind(sqlx::types::Uuid::parse_str(&uuid).unwrap())
    .execute(db)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    }
}

pub enum UpdateProofError {
    Sqlx(sqlx::Error),
    Io(io::Error),
}

pub async fn update_proof(
    uuid: &String,
    db: &sqlx::Pool<sqlx::Postgres>,
) -> Result<(), UpdateProofError> {
    let tmp_file_path = std::path::Path::new(&get_tmp_folder_path(&uuid)).join("proof.json");

    //remove the unwrap here later
    let proof_string = match std::fs::read_to_string(tmp_file_path) {
        Ok(proof_string) => proof_string,
        Err(e) => {
            dbg!(&e);
            return Err(UpdateProofError::Io(e));
        }
    };
    let mut proof_reader = serde_json::de::Deserializer::from_str(&proof_string);

    let proof = match Proof::deserialize(&mut proof_reader) {
        Ok(proof) => proof,
        Err(_) => {
            panic!("error");
        }
    };

    let status: i32 = types::Status::ProofGenererated.into();

    let now = Utc::now();
    match sqlx::query(
        "UPDATE proofs SET proof = $1, status = $2, proof_generated_at = $3  WHERE request_id = $4",
    )
    .bind(sqlx::types::Json(proof))
    .bind(status)
    .bind(now)
    .bind(sqlx::types::Uuid::parse_str(&uuid).unwrap())
    .execute(db)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            dbg!(&e);
            return Err(UpdateProofError::Sqlx(e));
        }
    }
}

pub async fn fail_proof(uuid: &String, db: &sqlx::Pool<sqlx::Postgres>) -> Result<(), sqlx::Error> {
    let status: i32 = types::Status::Failed.into();
    match sqlx::query("UPDATE proofs SET status = $1 WHERE request_id = $2")
        .bind(status)
        .bind(sqlx::types::Uuid::parse_str(&uuid).unwrap())
        .execute(db)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Proof {
    pi_a: Vec<String>,
    pi_b: Vec<Vec<String>>,
    pi_c: Vec<String>,
    protocol: String,
}
