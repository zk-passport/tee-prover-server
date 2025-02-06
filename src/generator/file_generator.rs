use std::path;

use crate::utils::get_tmp_folder_path;

use crate::types::{ProofRequest, ProofType};
use tokio::io::AsyncWriteExt;

pub struct FileGenerator {
    uuid: String,
    proof_request: ProofRequest,
}

impl FileGenerator {
    pub fn new(uuid: String, proof_request: ProofRequest) -> Self {
        Self {
            uuid,
            proof_request,
        }
    }

    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }

    pub fn proof_type(&self) -> ProofType {
        (&self.proof_request).into()
    }

    //create the tmp folder
    //create the inputs file
    //create the public_inputs file
    pub async fn run(&self) -> Result<(String, String), std::io::Error> {
        let path_str = get_tmp_folder_path(&self.uuid);
        let path = path::Path::new(&path_str);
        // let _ = tokio::fs::create_dir_all(path).await.unwrap();
        tokio::fs::create_dir_all(path).await?;

        let mut input_file = tokio::fs::File::create(path.join("input.json")).await?;

        input_file
            .write(self.proof_request.circuit().inputs.as_bytes())
            .await?;

        let mut public_input_file =
            tokio::fs::File::create(path.join("public_inputs.json")).await?;

        public_input_file
            .write(self.proof_request.circuit().public_inputs.as_bytes())
            .await?;

        Ok((self.uuid.clone(), self.proof_request.circuit().name.clone()))
    }
}
