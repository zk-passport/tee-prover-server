use std::{collections::HashMap, sync::Arc};

use jsonrpsee::ResponsePayload;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{db::StatusUpdatePayload, generator::Circuit};

#[derive(Serialize, Clone)]
pub struct HelloResponse {
    uuid: uuid::Uuid,
    pubkey: Vec<u8>,
}

impl HelloResponse {
    pub fn new(uuid: uuid::Uuid, pubkey: Vec<u8>) -> Self {
        HelloResponse { uuid, pubkey }
    }
}

impl<'a> Into<ResponsePayload<'a, HelloResponse>> for HelloResponse {
    fn into(self) -> ResponsePayload<'a, HelloResponse> {
        ResponsePayload::success(self)
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProofRequest {
    #[serde(rename_all = "camelCase")]
    Register { prove: Circuit },
    // Register { prove: Circuit, dsc: Circuit },
    #[serde(rename_all = "camelCase")]
    Disclose { disclose: Circuit },
}

pub type ConnectionMap =
    Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<StatusUpdatePayload>>>>;
