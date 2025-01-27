use std::path;

use crate::utils::get_tmp_folder_path;

use super::{Circuit, ProofType};
use tokio::io::AsyncWriteExt;

pub struct FileGenerator {
    uuid: String,
    proof_type: ProofType,
    circuit: Circuit,
}

impl FileGenerator {
    pub fn new(uuid: String, proof_type: ProofType, circuit: Circuit) -> Self {
        Self {
            uuid,
            proof_type,
            circuit,
        }
    }

    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }

    pub fn proof_type(&self) -> ProofType {
        self.proof_type.clone()
    }

    //create the tmp folder
    //create the inputs file
    //create the public_inputs file
    pub async fn run(&self) -> Result<(String, ProofType, String), std::io::Error> {
        let path_str = get_tmp_folder_path(&self.uuid, &self.proof_type);
        let path = path::Path::new(&path_str);
        let _ = tokio::fs::create_dir_all(path).await.unwrap();

        let mut input_file = tokio::fs::File::create(path.join("input.json")).await?;

        input_file.write(self.circuit.inputs.as_bytes()).await?;

        let mut public_input_file =
            tokio::fs::File::create(path.join("public_inputs.json")).await?;

        public_input_file
            .write(self.circuit.public_inputs.as_bytes())
            .await?;

        Ok((
            self.uuid.clone(),
            self.proof_type.clone(),
            self.circuit.name.clone(),
        ))
    }
}
