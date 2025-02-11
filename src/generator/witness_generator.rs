use core::str;
use std::path;

use crate::utils::get_tmp_folder_path;

pub struct WitnessGenerator {
    pub uuid: String,
    circuit_file_name: String,
}

impl WitnessGenerator {
    pub fn new(uuid: String, circuit_file_name: String) -> Self {
        WitnessGenerator {
            uuid,
            circuit_file_name,
        }
    }

    pub async fn run(
        &self,
        circuit_folder: &str, //folder where all the circuit executables are
    ) -> Result<(String, String), String> {
        let circuit_folder_path = path::Path::new(&circuit_folder);
        //TODO: covnert circuit_file_name to camel case if in snake case?
        let path = circuit_folder_path
            .join(format!("{}_cpp", &self.circuit_file_name))
            .join(&self.circuit_file_name);

        if !path.exists() {
            println!("{:?} does not exist", &path);
            return Err(format!("Circuit not found: {}", path.to_str().unwrap()));
        }

        let circuit_exe = format!("./{}", path.into_os_string().into_string().unwrap());
        let tmp_folder_path = get_tmp_folder_path(&self.uuid);
        let input_file = tmp_folder_path.clone() + "/input.json";
        let output_file = tmp_folder_path + "/output.wtns";

        let _ = match tokio::process::Command::new(circuit_exe)
            .arg(&input_file)
            .arg(&output_file)
            .output()
            .await
        {
            Ok(output) => {
                if !output.status.success() || output.stderr.len() > 0 {
                    let str = str::from_utf8(&output.stderr).unwrap();
                    return Err(str.to_string());
                }
            }
            Err(err) => {
                return Err(err.to_string());
            }
        };

        Ok((self.uuid.clone(), self.circuit_file_name.clone()))
    }
}
