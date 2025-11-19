#![allow(non_snake_case)]

use std::collections::HashMap;

use error_messages::{
    NO_ADDRESSES_AVAILABLE, NO_KNOWN_CHAIN_CONFIG_SC, NO_KNOWN_CHAIN_FACTORY_SC,
    NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID, NO_KNOWN_DYNAMIC_NFT_TOKEN_ID,
    NO_KNOWN_DYNAMIC_SFT_TOKEN_ID, NO_KNOWN_FEE_MARKET, NO_KNOWN_FEE_TOKEN, NO_KNOWN_FIRST_TOKEN,
    NO_KNOWN_FUNGIBLE_TOKEN, NO_KNOWN_HEADER_VERIFIER, NO_KNOWN_META_ESDT_TOKEN,
    NO_KNOWN_MVX_ESDT_SAFE, NO_KNOWN_NFT_TOKEN, NO_KNOWN_SFT_TOKEN, NO_KNOWN_SOVEREIGN_FORGE_SC,
    NO_KNOWN_TESTING_SC, NO_KNOWN_TRUSTED_TOKEN,
};
use multiversx_sc::imports::Bech32Address;
use multiversx_sc_snippets::imports::*;

#[derive(Debug, Clone)]
pub struct EsdtTokenInfo {
    pub token_id: EgldOrEsdtTokenIdentifier<StaticApi>,
    pub nonce: u64,
    pub token_type: EsdtTokenType,
    pub decimals: usize,
    pub amount: BigUint<StaticApi>,
}

#[derive(Debug, Clone)]
pub struct AddressInfo {
    pub address: Bech32Address,
    pub chain_id: String,
}

// NOTE: This struct holds deployed contract addresses.
// The index of each address corresponds to the shard number where the contract was deployed.
// For example, index 0 = shard 0, index 1 = shard 1, etc.
#[derive(Debug, Default)]
pub struct ShardAddresses {
    pub addresses: Vec<AddressInfo>,
}

impl ShardAddresses {
    pub fn push(&mut self, address: AddressInfo) -> usize {
        self.addresses.push(address);
        self.addresses.len() - 1
    }

    pub fn first(&self) -> &Bech32Address {
        &self
            .addresses
            .first()
            .expect(NO_ADDRESSES_AVAILABLE)
            .address
    }
}

#[derive(Debug, Default)]
pub struct State {
    pub fungible_tokens: Vec<EsdtTokenInfo>,
    pub fee_token: Option<EsdtTokenInfo>,
    pub nft_tokens: Vec<EsdtTokenInfo>,
    pub meta_esdt_tokens: Vec<EsdtTokenInfo>,
    pub dynamic_nft_tokens: Vec<EsdtTokenInfo>,
    pub dynamic_sft_tokens: Vec<EsdtTokenInfo>,
    pub dynamic_meta_esdt_tokens: Vec<EsdtTokenInfo>,
    pub sft_tokens: Vec<EsdtTokenInfo>,
    pub initial_wallet_tokens_state: Vec<EsdtTokenInfo>,
    pub trusted_token: Option<EsdtTokenInfo>,
    pub mvx_esdt_safe_addresses: Option<ShardAddresses>,
    pub header_verifier_addresses: Option<ShardAddresses>,
    pub fee_market_addresses: Option<ShardAddresses>,
    pub chain_config_sc_addresses: Option<ShardAddresses>,
    pub testing_sc_address: Option<Bech32Address>,
    pub sovereign_forge_sc_address: Option<Bech32Address>,
    pub chain_factory_sc_addresses: Option<Vec<Bech32Address>>,
    pub operation_nonce: HashMap<String, u64>,
    pub chain_ids: Vec<String>,
    pub bls_secret_keys: HashMap<String, Vec<Vec<u8>>>,
}

impl State {
    pub fn add_fungible_token(&mut self, token: EsdtTokenInfo) {
        self.fungible_tokens.push(token);
    }

    pub fn set_fee_token(&mut self, token: EsdtTokenInfo) {
        self.fee_token = Some(token);
    }

    pub fn add_nft_token(&mut self, token: EsdtTokenInfo) {
        self.nft_tokens.push(token);
    }

    pub fn add_meta_esdt_token(&mut self, token: EsdtTokenInfo) {
        self.meta_esdt_tokens.push(token);
    }

    pub fn add_dynamic_nft_token(&mut self, token: EsdtTokenInfo) {
        self.dynamic_nft_tokens.push(token);
    }

    pub fn add_sft_token(&mut self, token: EsdtTokenInfo) {
        self.sft_tokens.push(token);
    }

    pub fn add_dynamic_sft_token(&mut self, token: EsdtTokenInfo) {
        self.dynamic_sft_tokens.push(token);
    }

    pub fn add_dynamic_meta_esdt_token(&mut self, token: EsdtTokenInfo) {
        self.dynamic_meta_esdt_tokens.push(token);
    }

    pub fn update_or_add_initial_wallet_token(&mut self, token: EsdtTokenInfo) {
        if let Some(existing_token) = self
            .initial_wallet_tokens_state
            .iter_mut()
            .find(|t| t.token_id == token.token_id && t.nonce == token.nonce)
        {
            existing_token.amount += token.amount;
        } else {
            self.initial_wallet_tokens_state.push(token);
        }
    }

    pub fn set_trusted_token(&mut self, token: EsdtTokenInfo) {
        self.trusted_token = Some(token);
    }

    pub fn get_trusted_token(&self) -> EsdtTokenInfo {
        self.trusted_token
            .as_ref()
            .expect(NO_KNOWN_TRUSTED_TOKEN)
            .clone()
    }

    pub fn get_trusted_token_string(&self) -> String {
        self.trusted_token
            .as_ref()
            .expect(NO_KNOWN_TRUSTED_TOKEN)
            .token_id
            .clone()
            .into_managed_buffer()
            .to_string()
    }

    pub fn get_first_fungible_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.fungible_tokens
            .first()
            .expect(NO_KNOWN_FIRST_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_fee_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.fee_token
            .as_ref()
            .expect(NO_KNOWN_FEE_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_first_fungible_token_id(&self) -> EsdtTokenInfo {
        self.fungible_tokens
            .first()
            .expect(NO_KNOWN_FIRST_TOKEN)
            .clone()
    }

    pub fn get_fee_token_id(&self) -> EsdtTokenInfo {
        self.fee_token.as_ref().expect(NO_KNOWN_FEE_TOKEN).clone()
    }

    pub fn get_fungible_token_by_index(&self, index: usize) -> EsdtTokenInfo {
        self.fungible_tokens
            .get(index)
            .expect(NO_KNOWN_FUNGIBLE_TOKEN)
            .clone()
    }

    pub fn get_nft_token_by_index(&self, index: usize) -> EsdtTokenInfo {
        self.nft_tokens
            .get(index)
            .expect(NO_KNOWN_NFT_TOKEN)
            .clone()
    }

    pub fn get_meta_esdt_token_by_index(&self, index: usize) -> EsdtTokenInfo {
        self.meta_esdt_tokens
            .get(index)
            .expect(NO_KNOWN_META_ESDT_TOKEN)
            .clone()
    }

    pub fn get_dynamic_nft_token_by_index(&self, index: usize) -> EsdtTokenInfo {
        self.dynamic_nft_tokens
            .get(index)
            .expect(NO_KNOWN_DYNAMIC_NFT_TOKEN_ID)
            .clone()
    }

    pub fn get_sft_token_by_index(&self, index: usize) -> EsdtTokenInfo {
        self.sft_tokens
            .get(index)
            .expect(NO_KNOWN_SFT_TOKEN)
            .clone()
    }

    pub fn get_dynamic_sft_token_by_index(&self, index: usize) -> EsdtTokenInfo {
        self.dynamic_sft_tokens
            .get(index)
            .expect(NO_KNOWN_DYNAMIC_SFT_TOKEN_ID)
            .clone()
    }

    pub fn get_dynamic_meta_esdt_token_by_index(&self, index: usize) -> EsdtTokenInfo {
        self.dynamic_meta_esdt_tokens
            .get(index)
            .expect(NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID)
            .clone()
    }

    pub fn get_initial_wallet_tokens_state(&self) -> &Vec<EsdtTokenInfo> {
        &self.initial_wallet_tokens_state
    }

    pub fn get_initial_wallet_token_balance(
        &self,
        token_id: EgldOrEsdtTokenIdentifier<StaticApi>,
    ) -> BigUint<StaticApi> {
        self.initial_wallet_tokens_state
            .iter()
            .find(|token| token.token_id == token_id)
            .map_or_else(BigUint::zero, |token| token.amount.clone())
    }

    pub fn set_mvx_esdt_safe_contract_address(&mut self, address: AddressInfo) {
        let list = self.mvx_esdt_safe_addresses.get_or_insert_default();
        list.push(address);
    }

    pub fn set_header_verifier_address(&mut self, address: AddressInfo) {
        let list = self.header_verifier_addresses.get_or_insert_default();
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

    pub fn add_chain_id(&mut self, chain_id: String) {
        self.chain_ids.push(chain_id);
    }

    pub fn get_and_increment_operation_nonce(&mut self, contract_address: &str) -> u64 {
        let nonce = self.get_operation_nonce(contract_address);
        self.increment_operation_nonce(contract_address);
        nonce
    }

    /// Returns the contract addresses
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
        self.header_verifier_addresses
            .as_ref()
            .expect(NO_KNOWN_HEADER_VERIFIER)
            .addresses
            .get(shard as usize)
            .map(|info| &info.address)
            .unwrap_or_else(|| panic!("No Header Verifier address for shard {}", shard))
    }

    pub fn get_chain_id_for_shard(&self, shard: u32) -> &String {
        self.chain_ids
            .get(shard as usize)
            .unwrap_or_else(|| panic!("No chain ID for shard {}", shard))
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

    pub fn get_operation_nonce(&self, contract_address: &str) -> u64 {
        self.operation_nonce
            .get(contract_address)
            .copied()
            .unwrap_or(0)
    }

    pub fn increment_operation_nonce(&mut self, contract_address: &str) {
        let current_nonce = self
            .operation_nonce
            .entry(contract_address.to_string())
            .or_insert(0);
        *current_nonce += 1;
    }

    pub fn set_operation_nonce(&mut self, contract_address: &str, nonce: u64) {
        self.operation_nonce
            .entry(contract_address.to_string())
            .and_modify(|existing| *existing = nonce)
            .or_insert(nonce);
    }
}
