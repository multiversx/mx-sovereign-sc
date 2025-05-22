#![allow(non_snake_case)]

use multiversx_sc_snippets::imports::*;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

const STATE_FILE: &str = "state.toml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TokenProperties {
    pub token_id: String,
    pub nonce: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub mvx_esdt_safe_address: Option<Bech32Address>,
    pub header_verfier_address: Option<Bech32Address>,
    pub fee_market_address: Option<Bech32Address>,
    pub testing_sc_address: Option<Bech32Address>,
    pub chain_config_sc_address: Option<Bech32Address>,
    pub sovereign_forge_sc_address: Option<Bech32Address>,
    pub chain_factory_sc_address: Option<Bech32Address>,
    pub enshrine_esdt_safe_sc_address: Option<Bech32Address>,
    pub token_handler_address: Option<Bech32Address>,
    pub first_token: TokenProperties,
}

impl State {
    // Deserializes state from file
    pub fn load_state() -> Self {
        if Path::new(STATE_FILE).exists() {
            let mut file = std::fs::File::open(STATE_FILE).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Self::default()
        }
    }

    /// Sets the contract addresses
    pub fn set_mvx_esdt_safe_contract_address(&mut self, address: Bech32Address) {
        self.mvx_esdt_safe_address = Some(address);
    }

    pub fn set_header_verifier_address(&mut self, address: Bech32Address) {
        self.header_verfier_address = Some(address);
    }

    pub fn set_fee_market_address(&mut self, address: Bech32Address) {
        self.fee_market_address = Some(address);
    }

    pub fn set_testing_sc_address(&mut self, address: Bech32Address) {
        self.testing_sc_address = Some(address);
    }

    pub fn set_chain_config_sc_address(&mut self, address: Bech32Address) {
        self.chain_config_sc_address = Some(address);
    }

    pub fn set_sovereign_forge_sc_address(&mut self, address: Bech32Address) {
        self.sovereign_forge_sc_address = Some(address);
    }

    pub fn set_chain_factory_sc_address(&mut self, address: Bech32Address) {
        self.chain_factory_sc_address = Some(address);
    }

    pub fn set_enshrine_esdt_safe_sc_address(&mut self, address: Bech32Address) {
        self.enshrine_esdt_safe_sc_address = Some(address);
    }

    pub fn set_token_handler_address(&mut self, address: Bech32Address) {
        self.token_handler_address = Some(address);
    }

    pub fn set_first_token_id(&mut self, token: TokenProperties) {
        self.first_token = token;
    }

    /// Returns the contract addresses
    pub fn current_mvx_esdt_safe_contract_address(&self) -> &Bech32Address {
        self.mvx_esdt_safe_address
            .as_ref()
            .expect("no known contract, deploy first")
    }

    pub fn current_header_verifier_address(&self) -> &Bech32Address {
        self.header_verfier_address
            .as_ref()
            .expect("no known header verifier contract, deploy first")
    }

    pub fn current_fee_market_address(&self) -> &Bech32Address {
        self.fee_market_address
            .as_ref()
            .expect("no known fee market contract, deploy first")
    }

    pub fn current_testing_sc_address(&self) -> &Bech32Address {
        self.testing_sc_address
            .as_ref()
            .expect("no known testing SC contract, deploy first")
    }

    pub fn current_chain_config_sc_address(&self) -> &Bech32Address {
        self.chain_config_sc_address
            .as_ref()
            .expect("no known chain config SC contract, deploy first")
    }

    pub fn current_sovereign_forge_sc_address(&self) -> &Bech32Address {
        self.sovereign_forge_sc_address
            .as_ref()
            .expect("no known sovereign forge SC, deploy first")
    }

    pub fn current_chain_factory_sc_address(&self) -> &Bech32Address {
        self.chain_factory_sc_address
            .as_ref()
            .expect("no known chain factory SC, deploy first")
    }

    pub fn current_enshrine_esdt_safe_address(&self) -> &Bech32Address {
        self.enshrine_esdt_safe_sc_address
            .as_ref()
            .expect("no known enshrine esdt safe SC, deploy first")
    }

    pub fn current_token_handler_address(&self) -> &Bech32Address {
        self.token_handler_address
            .as_ref()
            .expect("no known token handler SC, deploy first")
    }
}

impl Drop for State {
    // Serializes state to file
    fn drop(&mut self) {
        let mut file = std::fs::File::create(STATE_FILE).unwrap();
        file.write_all(toml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}
