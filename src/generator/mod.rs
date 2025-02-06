pub mod file_generator;
pub mod proof_generator;
pub mod witness_generator;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Circuit {
    name: String,
    inputs: String,        //json
    public_inputs: String, //json
}
