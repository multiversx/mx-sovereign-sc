#![allow(non_snake_case)]

pub mod config;
pub mod mvx_esdt_safe;

use config::Config;
use multiversx_sc_snippets::imports::*;
use mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

const STATE_FILE: &str = "state.toml";

pub async fn mvx_esdt_safe_cli() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let config = Config::new();
    let mut interact = MvxEsdtSafeInteract::new(config).await;
    match cmd.as_str() {
        "upgrade" => interact.upgrade().await,
        "pause" => interact.pause_endpoint().await,
        "unpause" => interact.unpause_endpoint().await,
        "isPaused" => interact.paused_status().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub mvx_esdt_safe_address: Option<Bech32Address>,
    pub header_verfier_address: Option<Bech32Address>,
    pub fee_market_address: Option<Bech32Address>,
    pub testing_sc_address: Option<Bech32Address>,
    pub chain_config_sc_address: Option<Bech32Address>,
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
}

impl Drop for State {
    // Serializes state to file
    fn drop(&mut self) {
        let mut file = std::fs::File::create(STATE_FILE).unwrap();
        file.write_all(toml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}
