use serde::{Deserialize, Serialize};

use crate::{generator::ProofType, utils::get_tmp_folder_path};
pub mod types;

pub async fn create_proof_status(
    uuid: String,
    proof_type: &ProofType,
    status: types::Status,
    db: &sqlx::Pool<sqlx::Postgres>,
) -> Result<(), sqlx::Error> {
    let proof_status_id = proof_type.to_int() as i32;
    let status = status.to_int() as i32;
    match sqlx::query(
        "INSERT INTO proof_statuses (proof_status_id, request_id, status) VALUES ($1, $2, $3)",
    )
    .bind(proof_status_id)
    .bind(sqlx::types::Uuid::parse_str(&uuid).unwrap())
    .bind(status)
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

pub async fn update_proof_status(
    uuid: String,
    proof_type: &ProofType,
    status: types::Status,
    db: &sqlx::Pool<sqlx::Postgres>,
) -> Result<(), sqlx::Error> {
    let proof_status_id = proof_type.to_int() as i32;
    let status = status.to_int() as i32;

    match sqlx::query(
        "UPDATE proof_statuses SET status = $1 WHERE proof_status_id = $2 AND request_id = $3",
    )
    .bind(status)
    .bind(proof_status_id)
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

pub async fn update_proof(
    uuid: String,
    proof_type: &ProofType,
    // db: &sqlx::Pool<sqlx::Postgres>,
) -> Result<(), sqlx::Error> {
    let tmp_file_path =
        std::path::Path::new(&get_tmp_folder_path(&uuid, &proof_type)).join("proof.json");

    //remove the unwrap here later
    let proof_string = std::fs::read_to_string(tmp_file_path).unwrap();
    let mut proof = serde_json::de::Deserializer::from_str(&proof_string);

    match Proof::deserialize(&mut proof) {
        Ok(proof) => {
            dbg!(&proof);
        }
        Err(e) => {
            dbg!(&e);
        }
    };

    Ok(())
    // match sqlx::query(
    //     "UPDATE proof_statuses SET proof = $1, status = $2  WHERE request_id = $3 AND proof_status_id = $4",
    // )
    // .bind(sqlx::types::Json(proof))
    // .bind(types::Status::Completed.to_int() as i32)
    // .bind(sqlx::types::Uuid::parse_str(&uuid).unwrap())
    // .bind(proof_type.to_int() as i32)
    // .execute(db)
    // .await
    // {
    //     Ok(_) => Ok(()),
    //     Err(e) => {
    //         dbg!(&e);
    //         return Err(e);
    //     }
    // }
}

#[derive(Debug, Serialize, Deserialize)]
struct Proof {
    pi_a: Vec<String>,
    pi_b: Vec<Vec<String>>,
    pi_c: Vec<String>,
    protocol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusUpdatePayload {
    proof_status_id: i32,
    request_id: sqlx::types::Uuid,
    new_status: i32,
    proof: Option<sqlx::types::Json<Proof>>,
}

// pub async fn listen_status_update(
//     pool: &sqlx::Pool<sqlx::Postgres>,
//     channel: &str,
//     connection_map: crate::types::ConnectionMap,
// ) {
//     let mut listener = sqlx::postgres::PgListener::connect_with(pool)
//         .await
//         .unwrap();

//     listener.listen(channel).await.unwrap();

//     loop {
//         while let Ok(notification) = listener.recv().await {
//             let payload = notification.payload().to_owned();

//             let payload: StatusUpdatePayload = serde_json::from_str(&payload).unwrap();

//             let connection_map_rwlock = connection_map.read().await;

//             let status_update_sender =
//                 match connection_map_rwlock.get(&payload.request_id.to_string()) {
//                     Some(sender) => sender,
//                     None => continue,
//                 };

//             status_update_sender.send(payload).await.unwrap();
//         }
//     }
// }
