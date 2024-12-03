use multiversx_sc_snippets::imports::*;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

const STATE_FILE: &str = "state.toml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    esdt_safe_address: Option<Bech32Address>,
    header_verifier_address: Option<Bech32Address>,
    fee_market_address: Option<Bech32Address>,
    token_handler_address: Option<Bech32Address>,
    testing_sc_address: Option<Bech32Address>,
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

    /// Sets the contract address
    pub fn set_esdt_safe_address(&mut self, address: Bech32Address) {
        self.esdt_safe_address = Some(address);
    }

    pub fn set_header_verifier_address(&mut self, address: Bech32Address) {
        self.header_verifier_address = Some(address);
    }

    pub fn set_fee_market_address(&mut self, address: Bech32Address) {
        self.fee_market_address = Some(address);
    }

    pub fn set_token_handler_address(&mut self, address: Bech32Address) {
        self.token_handler_address = Some(address);
    }

    pub fn set_testing_sc_address(&mut self, address: Bech32Address) {
        self.testing_sc_address = Some(address);
    }

    /// Returns the contract address
    pub fn esdt_safe_address(&self) -> &Bech32Address {
        self.esdt_safe_address
            .as_ref()
            .expect("no known esdt_safe contract, deploy first")
    }

    pub fn get_header_verifier_address(&self) -> &Bech32Address {
        self.header_verifier_address
            .as_ref()
            .expect("no known header verifier, deploy first")
    }

    pub fn get_fee_market_address(&self) -> &Bech32Address {
        self.fee_market_address
            .as_ref()
            .expect("no known fee market, deploy first")
    }

    pub fn get_token_handler_address(&self) -> &Bech32Address {
        self.token_handler_address
            .as_ref()
            .expect("no known token handler, deploy first")
    }

    pub fn get_testing_sc_address(&self) -> Address {
        self.testing_sc_address.clone().unwrap().to_address()
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
