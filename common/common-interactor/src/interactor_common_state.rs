use std::{
    collections::HashMap,
    io::{Read, Write},
    path::Path,
};

use common_test_setup::constants::STATE_FILE;
use error_messages::{
    NO_KNOWN_CHAIN_CONFIG_SC, NO_KNOWN_CHAIN_FACTORY_IN_THE_SPECIFIED_SHARD,
    NO_KNOWN_CHAIN_FACTORY_SC, NO_KNOWN_FEE_MARKET, NO_KNOWN_FEE_TOKEN, NO_KNOWN_HEADER_VERIFIER,
    NO_KNOWN_MVX_ESDT_SAFE, NO_KNOWN_SOVEREIGN_FORGE_SC, NO_KNOWN_TESTING_SC,
};
use multiversx_sc::{
    imports::Bech32Address,
    types::{BigUint, EgldOrEsdtTokenIdentifier, EsdtTokenType},
};
use serde::{Deserialize, Serialize};

use crate::{
    interactor_state::{AddressInfo, EsdtTokenInfo, ShardAddresses},
    interactor_structs::SerializableFeeMarketToken,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CommonState {
    pub mvx_esdt_safe_addresses: Option<ShardAddresses>,
    pub header_verfier_addresses: Option<ShardAddresses>,
    pub fee_market_addresses: Option<ShardAddresses>,
    pub chain_config_sc_addresses: Option<ShardAddresses>,
    pub testing_sc_address: Option<Bech32Address>,
    pub sovereign_forge_sc_address: Option<Bech32Address>,
    pub chain_factory_sc_addresses: Option<Vec<Bech32Address>>,
    pub fee_market_tokens: HashMap<String, SerializableFeeMarketToken>,
    pub fee_status: HashMap<String, bool>,
    pub fee_op_nonce: u64,
    pub chain_ids: Vec<String>,
    pub update_config_nonce: u64,
    pub mvx_egld_balances: Vec<(String, u64)>,
    pub testing_egld_balance: u64,
    pub bls_secret_keys: HashMap<String, Vec<Vec<u8>>>,
}

impl CommonState {
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

    pub fn set_mvx_esdt_safe_contract_address(&mut self, address: AddressInfo) {
        let list = self.mvx_esdt_safe_addresses.get_or_insert_default();
        list.push(address);
    }

    pub fn set_header_verifier_address(&mut self, address: AddressInfo) {
        let list = self.header_verfier_addresses.get_or_insert_default();
        list.push(address);
    }

    pub fn set_fee_market_address(&mut self, address: AddressInfo) {
        let list = self.fee_market_addresses.get_or_insert_default();
        list.push(address);
    }

    pub fn set_chain_config_sc_address(&mut self, address: AddressInfo) {
        let list = self.chain_config_sc_addresses.get_or_insert_default();
        list.push(address);
    }

    pub fn set_testing_sc_address(&mut self, address: Bech32Address) {
        self.testing_sc_address = Some(address);
    }

    pub fn set_sovereign_forge_sc_address(&mut self, address: Bech32Address) {
        self.sovereign_forge_sc_address = Some(address);
    }

    pub fn set_chain_factory_sc_address(&mut self, address: Bech32Address) {
        let list = self.chain_factory_sc_addresses.get_or_insert_default();
        list.push(address);
    }

    pub fn set_fee_status_for_shard(&mut self, shard: u32, status: bool) {
        self.fee_status.insert(shard.to_string(), status);
    }

    pub fn set_fee_status_for_all_shards(&mut self, status: bool) {
        for shard in 0..3 {
            self.fee_status.insert(shard.to_string(), status);
        }
    }

    pub fn set_fee_market_token_for_all_shards(&mut self, token: SerializableFeeMarketToken) {
        for shard in 0..3 {
            self.fee_market_tokens
                .insert(shard.to_string(), token.clone());
        }
    }

    pub fn set_fee_market_token_for_shard(
        &mut self,
        shard: u32,
        token: SerializableFeeMarketToken,
    ) {
        self.fee_market_tokens.insert(shard.to_string(), token);
    }

    pub fn add_chain_id(&mut self, chain_id: String) {
        self.chain_ids.push(chain_id);
    }

    pub fn set_mvx_egld_balance_for_all_shards(&mut self, balance: u64) {
        for shard in 0..3 {
            self.mvx_egld_balances.push((shard.to_string(), balance));
        }
    }

    pub fn update_mvx_egld_balance_with_amount(&mut self, shard: u32, amount: u64) {
        // Find existing balance for shard and add to it
        let shard_str = shard.to_string();
        if let Some((_, current_balance)) = self
            .mvx_egld_balances
            .iter_mut()
            .find(|(s, _)| s == &shard_str)
        {
            *current_balance += amount;
        }
    }

    pub fn update_testing_egld_balance_with_amount(&mut self, amount: u64) {
        self.testing_egld_balance += amount;
    }

    /// Returns the contract addresses
    pub fn current_mvx_esdt_safe_contract_address(&self) -> &Bech32Address {
        self.mvx_esdt_safe_addresses
            .as_ref()
            .expect(NO_KNOWN_MVX_ESDT_SAFE)
            .first()
    }

    pub fn current_header_verifier_address(&self) -> &Bech32Address {
        self.header_verfier_addresses
            .as_ref()
            .expect(NO_KNOWN_HEADER_VERIFIER)
            .first()
    }

    pub fn current_fee_market_address(&self) -> &Bech32Address {
        self.fee_market_addresses
            .as_ref()
            .expect(NO_KNOWN_FEE_MARKET)
            .first()
    }

    pub fn current_chain_config_sc_address(&self) -> &Bech32Address {
        self.chain_config_sc_addresses
            .as_ref()
            .expect(NO_KNOWN_CHAIN_CONFIG_SC)
            .first()
    }

    pub fn current_testing_sc_address(&self) -> &Bech32Address {
        self.testing_sc_address.as_ref().expect(NO_KNOWN_TESTING_SC)
    }

    pub fn current_sovereign_forge_sc_address(&self) -> &Bech32Address {
        self.sovereign_forge_sc_address
            .as_ref()
            .expect(NO_KNOWN_SOVEREIGN_FORGE_SC)
    }

    pub fn current_chain_factory_sc_address(&self) -> &Bech32Address {
        self.chain_factory_sc_addresses
            .as_ref()
            .expect(NO_KNOWN_CHAIN_FACTORY_SC)
            .first()
            .expect(NO_KNOWN_CHAIN_FACTORY_IN_THE_SPECIFIED_SHARD)
    }

    pub fn get_chain_factory_sc_address(&self, shard: u32) -> &Bech32Address {
        self.chain_factory_sc_addresses
            .as_ref()
            .expect(NO_KNOWN_CHAIN_FACTORY_SC)
            .get(shard as usize)
            .unwrap_or_else(|| panic!("No Chain Factory SC address for shard {}", shard))
    }

    pub fn get_mvx_esdt_safe_address(&self, shard: u32) -> &Bech32Address {
        self.mvx_esdt_safe_addresses
            .as_ref()
            .expect(NO_KNOWN_MVX_ESDT_SAFE)
            .addresses
            .get(shard as usize)
            .map(|info| &info.address)
            .unwrap_or_else(|| panic!("No MVX ESDT Safe address for shard {}", shard))
    }

    pub fn get_fee_market_address(&self, shard: u32) -> &Bech32Address {
        self.fee_market_addresses
            .as_ref()
            .expect(NO_KNOWN_FEE_MARKET)
            .addresses
            .get(shard as usize)
            .map(|info| &info.address)
            .unwrap_or_else(|| panic!("No Fee Market address for shard {}", shard))
    }

    pub fn get_header_verifier_address(&self, shard: u32) -> &Bech32Address {
        self.header_verfier_addresses
            .as_ref()
            .expect(NO_KNOWN_HEADER_VERIFIER)
            .addresses
            .get(shard as usize)
            .map(|info| &info.address)
            .unwrap_or_else(|| panic!("No Header Verifier address for shard {}", shard))
    }

    pub fn get_fee_status_for_shard(&self, shard: u32) -> bool {
        self.fee_status
            .get(&shard.to_string())
            .cloned()
            .unwrap_or(false)
    }

    pub fn get_fee_market_token_amount_for_shard(&self, shard: u32) -> u64 {
        self.fee_market_tokens
            .get(&shard.to_string())
            .cloned()
            .expect(NO_KNOWN_FEE_TOKEN)
            .amount
    }

    pub fn get_fee_market_token_for_shard_converted(&self, shard: u32) -> EsdtTokenInfo {
        let token = self
            .fee_market_tokens
            .get(&shard.to_string())
            .cloned()
            .expect(NO_KNOWN_FEE_TOKEN);
        EsdtTokenInfo {
            token_id: EgldOrEsdtTokenIdentifier::from(token.token_id.as_str()),
            nonce: token.nonce,
            token_type: EsdtTokenType::from(token.token_type),
            decimals: token.decimals,
            amount: BigUint::from(token.amount),
        }
    }

    pub fn get_fee_market_token_for_shard(&self, shard: u32) -> SerializableFeeMarketToken {
        self.fee_market_tokens
            .get(&shard.to_string())
            .cloned()
            .expect(NO_KNOWN_FEE_TOKEN)
    }

    pub fn get_chain_id_for_shard(&self, shard: u32) -> &String {
        self.chain_ids
            .get(shard as usize)
            .unwrap_or_else(|| panic!("No chain ID for shard {}", shard))
    }

    pub fn get_update_config_nonce(&self) -> u64 {
        self.update_config_nonce
    }

    pub fn get_mvx_egld_balance_for_shard(&self, shard: u32) -> u64 {
        self.mvx_egld_balances
            .get(shard as usize)
            .map(|(_, balance)| *balance)
            .unwrap_or(0u64)
    }

    pub fn get_testing_egld_balance(&self) -> u64 {
        self.testing_egld_balance
    }

    pub fn add_bls_secret_key(&mut self, shard: u32, secret_key_bytes: Vec<u8>) {
        let shard_key = shard.to_string();
        self.bls_secret_keys
            .entry(shard_key)
            .or_default()
            .push(secret_key_bytes);
    }

    pub fn get_bls_secret_keys(&self, shard: u32) -> Option<&Vec<Vec<u8>>> {
        let shard_key = shard.to_string();
        self.bls_secret_keys.get(&shard_key)
    }
}

impl Drop for CommonState {
    // Serializes state to file
    fn drop(&mut self) {
        let mut file = std::fs::File::create(STATE_FILE).unwrap();
        file.write_all(toml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}
