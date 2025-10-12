use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub external_services: ExternalServices,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalServices {
    pub plc_directory: String,
    pub s3_endpoint: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let plc_directory =
            env::var("PLC_DIRECTORY").unwrap_or("https://plc.directory".to_string());
        let server_port = env::var("SERVER_PORT").unwrap_or("9090".to_string());
        let worker_count = env::var("WORKER_COUNT").unwrap_or("2".to_string());
        let s3_endpoint = env::var("ENDPOINT").expect("ENDPOINT environment variable not set");

        Self {
            server: ServerConfig {
                port: server_port.parse().unwrap(),
                workers: worker_count.parse().unwrap(),
            },
            external_services: ExternalServices {
                plc_directory,
                s3_endpoint,
            },
        }
    }
}
