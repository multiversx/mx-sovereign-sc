#![allow(non_snake_case)]

use error_messages::{
    NO_KNOWN_CHAIN_CONFIG_SC, NO_KNOWN_CHAIN_FACTORY_SC, NO_KNOWN_FEE_MARKET, NO_KNOWN_FEE_TOKEN,
    NO_KNOWN_FIRST_TOKEN, NO_KNOWN_HEADER_VERIFIER, NO_KNOWN_MVX_ESDT_SAFE, NO_KNOWN_SECOND_TOKEN,
    NO_KNOWN_SOVEREIGN_FORGE_SC, NO_KNOWN_TESTING_SC,
};
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
    pub chain_factory_sc_address: Option<Vec<Bech32Address>>,
    pub first_token: Option<TokenProperties>,
    pub fee_token: Option<TokenProperties>,
    pub second_token: Option<TokenProperties>,
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

    pub fn set_chain_factory_sc_address_for_shard(&mut self, address: Bech32Address) {
        self.chain_factory_sc_address
            .get_or_insert_with(Vec::new)
            .push(address);
    }

    pub fn set_first_token(&mut self, token: TokenProperties) {
        self.first_token = Some(token);
    }

    pub fn set_fee_token(&mut self, token: TokenProperties) {
        self.fee_token = Some(token);
    }

    pub fn set_second_token(&mut self, token: TokenProperties) {
        self.second_token = Some(token);
    }

    /// Returns the contract addresses
    pub fn current_mvx_esdt_safe_contract_address(&self) -> &Bech32Address {
        self.mvx_esdt_safe_address
            .as_ref()
            .expect(NO_KNOWN_MVX_ESDT_SAFE)
    }

    pub fn current_header_verifier_address(&self) -> &Bech32Address {
        self.header_verfier_address
            .as_ref()
            .expect(NO_KNOWN_HEADER_VERIFIER)
    }

    pub fn current_fee_market_address(&self) -> &Bech32Address {
        self.fee_market_address.as_ref().expect(NO_KNOWN_FEE_MARKET)
    }

    pub fn current_testing_sc_address(&self) -> &Bech32Address {
        self.testing_sc_address.as_ref().expect(NO_KNOWN_TESTING_SC)
    }

    pub fn current_chain_config_sc_address(&self) -> &Bech32Address {
        self.chain_config_sc_address
            .as_ref()
            .expect(NO_KNOWN_CHAIN_CONFIG_SC)
    }

    pub fn current_sovereign_forge_sc_address(&self) -> &Bech32Address {
        self.sovereign_forge_sc_address
            .as_ref()
            .expect(NO_KNOWN_SOVEREIGN_FORGE_SC)
    }

    pub fn current_chain_factory_sc_address(&self) -> &Bech32Address {
        self.chain_factory_sc_address
            .as_ref()
            .and_then(|v| v.first())
            .expect(NO_KNOWN_CHAIN_FACTORY_SC)
    }

    pub fn get_chain_factory_address_for_shard(&self, shard: u32) -> &Bech32Address {
        self.chain_factory_sc_address
            .as_ref()
            .and_then(|v| v.get(shard as usize))
            .expect(NO_KNOWN_CHAIN_FACTORY_SC)
    }

    pub fn get_first_token_id_string(&self) -> String {
        self.first_token
            .as_ref()
            .expect(NO_KNOWN_FIRST_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_fee_token_id_string(&self) -> String {
        self.fee_token
            .as_ref()
            .expect(NO_KNOWN_FEE_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_second_token_id_string(&self) -> String {
        self.second_token
            .as_ref()
            .expect(NO_KNOWN_SECOND_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_first_token_id(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.first_token
            .as_ref()
            .expect(NO_KNOWN_FIRST_TOKEN)
            .token_id
            .as_str()
            .into()
    }

    pub fn get_fee_token_id(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.fee_token
            .as_ref()
            .expect(NO_KNOWN_FEE_TOKEN)
            .token_id
            .as_str()
            .into()
    }

    pub fn get_second_token_id(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.second_token
            .as_ref()
            .expect(NO_KNOWN_SECOND_TOKEN)
            .token_id
            .as_str()
            .into()
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
