use std::str::FromStr;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Config {
    /// Web server bind address (e.g., 0.0.0.0:3001)
    #[arg(short, long, default_value = "0.0.0.0:3001")]
    pub server_address: String,

    /// WebSocket server URL (e.g., ws://0.0.0.0:3002)
    #[arg(short, long, default_value = "0.0.0.0:3002")]
    pub ws_server_url: String,

    /// PostgreSQL database connection URL
    #[arg(
        short,
        long,
        default_value = "postgres://postgres:mysecretpassword@localhost:5433/db"
    )]
    pub database_url: String,

    /// Circuit folder path
    #[arg(short = 'c', long, default_value = "../circuits")]
    pub circuit_folder: String,

    /// Circuit file prefix
    #[arg(short = 'p', long, default_value = "build_witnesscalc_")]
    pub circuit_file_prefix: String,

    /// ZKey folder path
    #[arg(short = 'k', long, default_value = "./zkeys")]
    pub zkey_folder: String,

    /// Witness calc circuit to zkey mapper
    #[arg(short = 'z', long)]
    pub circuit_zkey_map: Vec<KeyValuePair>,
}

#[derive(Debug, Clone)]
pub struct KeyValuePair(pub String, pub String);

impl FromStr for KeyValuePair {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((key, value)) = s.split_once('=') {
            Ok(KeyValuePair(key.to_string(), value.to_string()))
        } else {
            Err("Expected format: key=value")
        }
    }
}
