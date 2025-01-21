use std::path;

use crate::utils::get_tmp_folder_path;

use super::ProofType;

pub struct WitnessGenerator {
    uuid: String,
    proof_type: ProofType,
    circuit_file_name: String,
}

#[derive(Debug)]
pub enum WitnessGeneratorError {
    CircuitNotFound,
}

impl WitnessGenerator {
    pub fn new(uuid: String, proof_type: ProofType, circuit_file_name: String) -> Self {
        WitnessGenerator {
            uuid,
            proof_type,
            circuit_file_name,
        }
    }

    pub async fn run(
        &self,
        circuit_folder: &str,      //folder where all the circuit executables are
        circuit_file_prefix: &str, //executable circuit file prefix
    ) -> Result<(String, ProofType, String), WitnessGeneratorError> {
        let circuit_folder_path = path::Path::new(&circuit_folder);
        //TODO: covnert circuit_file_name to camel case if in snake case?
        let path = circuit_folder_path
            .join(format!(
                "{}{}",
                circuit_file_prefix, &self.circuit_file_name
            ))
            .join("src")
            .join(&self.circuit_file_name);

        if !path.exists() {
            println!("{:?} does not exist", &path);
            return Err(WitnessGeneratorError::CircuitNotFound);
        }

        let circuit_exe = format!("./{}", path.into_os_string().into_string().unwrap());
        dbg!(&circuit_exe);
        let tmp_folder_path = get_tmp_folder_path(&self.uuid, &self.proof_type);
        let input_file = tmp_folder_path.clone() + "/input.json";
        let output_file = tmp_folder_path + "/output.wtns";

        let _ = match tokio::process::Command::new(circuit_exe)
            .arg(input_file)
            .arg(output_file)
            .output()
            .await
        {
            Ok(output) => {
                dbg!(&self.uuid, &output);
            }
            Err(err) => {
                dbg!(err.to_string());
                return Err(WitnessGeneratorError::CircuitNotFound); //TODO: change the error
            }
        };

        Ok((
            self.uuid.clone(),
            self.proof_type.clone(),
            self.circuit_file_name.clone(),
        ))
    }
}
