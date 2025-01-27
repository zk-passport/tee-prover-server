use std::path;

use crate::utils::get_tmp_folder_path;

use super::ProofType;

pub struct ProofGenerator {
    uuid: String,
    proof_type: ProofType,
    zkey_file_path: String,
}

impl ProofGenerator {
    pub fn new(uuid: String, proof_type: ProofType, zkey_file_path: String) -> Self {
        ProofGenerator {
            uuid,
            proof_type,
            zkey_file_path,
        }
    }

    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }

    pub fn proof_type(&self) -> ProofType {
        self.proof_type.clone()
    }

    //TODO: check if all these files exist
    pub async fn run(&self, rapid_snark_path_exe: &String) {
        let witness_file_path_str = get_tmp_folder_path(&self.uuid, &self.proof_type);
        let witness_file_path = path::Path::new(&witness_file_path_str).join("output.wtns");

        let proof_file_path_str = get_tmp_folder_path(&self.uuid, &self.proof_type);
        let proof_file_path = path::Path::new(&proof_file_path_str).join("proof.json");

        let public_inputs = get_tmp_folder_path(&self.uuid, &self.proof_type);
        let public_inputs = path::Path::new(&public_inputs).join("public_inputs.json");

        let _ = match tokio::process::Command::new(format!("./{}", rapid_snark_path_exe))
            .arg(&self.zkey_file_path)
            .arg(witness_file_path)
            .arg(proof_file_path)
            .arg(public_inputs.into_os_string().into_string().unwrap())
            .output()
            .await
        {
            Ok(output) => {
                // dbg!(&self.uuid, &output);
            }
            Err(err) => {
                dbg!(err.to_string());
            }
        };
    }
}
