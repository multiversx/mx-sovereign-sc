#![allow(unused)]

use serde::Deserialize;
use std::io::Read;

/// Config file
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    Real,
    Simulator,
}

/// Contract Interact configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub gateway_uri: String,
    pub chain_type: ChainType,
}

impl Config {
    // Deserializes config from file
    pub fn load_config() -> Self {
        let mut file = std::fs::File::open(CONFIG_FILE).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        toml::from_str(&content).unwrap()
    }

    pub fn chain_simulator_config() -> Self {
        let port = std::env::var("CHAIN_SIMULATOR_PORT").unwrap_or_else(|_| "8085".to_string());
        Config {
            gateway_uri: format!("http://localhost:{}", port),
            chain_type: ChainType::Simulator,
        }
    }

    // Returns the gateway URI
    pub fn gateway_uri(&self) -> &str {
        &self.gateway_uri
    }

    // Returns if chain type is chain simulator
    pub fn use_chain_simulator(&self) -> bool {
        match self.chain_type {
            ChainType::Real => false,
            ChainType::Simulator => true,
        }
    }
}
