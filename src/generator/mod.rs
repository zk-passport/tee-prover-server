pub mod file_generator;
pub mod proof_generator;
pub mod witness_generator;

use serde::Deserialize;

#[derive(Clone)]
pub enum ProofType {
    Prove,
    Dsc,
    Disclose,
}

impl ProofType {
    pub fn to_int(&self) -> u8 {
        match self {
            ProofType::Prove => 0,
            ProofType::Dsc => 1,
            ProofType::Disclose => 2,
        }
    }
}

impl std::fmt::Display for ProofType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProofType::Prove => write!(f, "prove"),
            ProofType::Dsc => write!(f, "dsc"),
            ProofType::Disclose => write!(f, "disclose"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Circuit {
    name: String,
    inputs: String,        //json
    public_inputs: String, //json
}
