#![allow(non_snake_case)]

use error_messages::{
    NO_KNOWN_CHAIN_CONFIG_SC, NO_KNOWN_CHAIN_FACTORY_IN_THE_SPECIFIED_SHARD,
    NO_KNOWN_CHAIN_FACTORY_SC, NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID, NO_KNOWN_DYNAMIC_NFT_TOKEN_ID,
    NO_KNOWN_DYNAMIC_SFT_TOKEN_ID, NO_KNOWN_ENSHRINE_ESDT_SAFE_SC, NO_KNOWN_FEE_MARKET,
    NO_KNOWN_FEE_TOKEN, NO_KNOWN_FIRST_TOKEN, NO_KNOWN_HEADER_VERIFIER, NO_KNOWN_META_ESDT_TOKEN,
    NO_KNOWN_MVX_ESDT_SAFE, NO_KNOWN_NFT_TOKEN, NO_KNOWN_SECOND_TOKEN, NO_KNOWN_SFT_TOKEN,
    NO_KNOWN_SOVEREIGN_FORGE_SC, NO_KNOWN_SOV_TO_MVX_TOKEN, NO_KNOWN_TESTING_SC,
    NO_KNOWN_TOKEN_HANDLER_IN_THE_SPECIFIED_SHARD, NO_KNOWN_TOKEN_HANDLER_SC,
};
use multiversx_sc_snippets::imports::*;

#[derive(Debug, Clone)]
pub struct EsdtTokenInfo {
    pub token_id: String,
    pub nonce: u64,
    pub token_type: EsdtTokenType,
}

#[derive(Debug)]
pub struct AddressInfo {
    pub address: Bech32Address,
    pub chain_id: String,
}

#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub token_id: String,
    pub amount: BigUint<StaticApi>,
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
            .expect("No addresses available")
            .address
    }
}

#[derive(Debug, Default)]
pub struct State {
    pub mvx_esdt_safe_addresses: Option<ShardAddresses>,
    pub header_verfier_addresses: Option<ShardAddresses>,
    pub fee_market_addresses: Option<ShardAddresses>,
    pub testing_sc_address: Option<Bech32Address>,
    pub chain_config_sc_addresses: Option<ShardAddresses>,
    pub sovereign_forge_sc_addresses: Option<Bech32Address>,
    pub chain_factory_sc_addresses: Option<Vec<Bech32Address>>,
    pub enshrine_esdt_safe_sc_addresses: Option<ShardAddresses>,
    pub token_handler_addresses: Option<Vec<Bech32Address>>,
    pub first_token: Option<EsdtTokenInfo>,
    pub fee_token: Option<EsdtTokenInfo>,
    pub second_token: Option<EsdtTokenInfo>,
    pub nft_token_id: Option<EsdtTokenInfo>,
    pub meta_esdt_token_id: Option<EsdtTokenInfo>,
    pub dynamic_nft_token_id: Option<EsdtTokenInfo>,
    pub dynamic_sft_token_id: Option<EsdtTokenInfo>,
    pub dynamic_meta_esdt_token_id: Option<EsdtTokenInfo>,
    pub sft_token_id: Option<EsdtTokenInfo>,
    pub sov_to_mvx_token_id: Option<EsdtTokenInfo>,
    pub initial_balance: Vec<(Bech32Address, Vec<TokenBalance>)>,
}

impl State {
    /// Sets the contract addresses
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

    pub fn set_testing_sc_address(&mut self, address: Bech32Address) {
        self.testing_sc_address = Some(address);
    }

    pub fn set_chain_config_sc_address(&mut self, address: AddressInfo) {
        let list = self.chain_config_sc_addresses.get_or_insert_default();
        list.push(address);
    }

    pub fn set_sovereign_forge_sc_address(&mut self, address: Bech32Address) {
        self.sovereign_forge_sc_addresses = Some(address);
    }

    pub fn set_chain_factory_sc_address(&mut self, address: Bech32Address) {
        let list = self.chain_factory_sc_addresses.get_or_insert_default();
        list.push(address);
    }

    pub fn set_enshrine_esdt_safe_sc_address(&mut self, address: AddressInfo) {
        let list = self.enshrine_esdt_safe_sc_addresses.get_or_insert_default();
        list.push(address);
    }

    pub fn set_token_handler_address(&mut self, address: Bech32Address) {
        let list = self.token_handler_addresses.get_or_insert_default();
        list.push(address);
    }

    pub fn set_first_token(&mut self, token: EsdtTokenInfo) {
        self.first_token = Some(token);
    }

    pub fn set_fee_token(&mut self, token: EsdtTokenInfo) {
        self.fee_token = Some(token);
    }

    pub fn set_second_token(&mut self, token: EsdtTokenInfo) {
        self.second_token = Some(token);
    }

    pub fn set_nft_token_id(&mut self, token: EsdtTokenInfo) {
        self.nft_token_id = Some(token);
    }

    pub fn set_meta_esdt_token_id(&mut self, token: EsdtTokenInfo) {
        self.meta_esdt_token_id = Some(token);
    }

    pub fn set_dynamic_nft_token_id(&mut self, token: EsdtTokenInfo) {
        self.dynamic_nft_token_id = Some(token);
    }

    pub fn set_sft_token_id(&mut self, token: EsdtTokenInfo) {
        self.sft_token_id = Some(token);
    }

    pub fn set_dynamic_sft_token_id(&mut self, token: EsdtTokenInfo) {
        self.dynamic_sft_token_id = Some(token);
    }

    pub fn set_dynamic_meta_esdt_token_id(&mut self, token: EsdtTokenInfo) {
        self.dynamic_meta_esdt_token_id = Some(token);
    }

    pub fn set_sov_to_mvx_token_id(&mut self, token: EsdtTokenInfo) {
        self.sov_to_mvx_token_id = Some(token);
    }

    pub fn set_initial_balance(&mut self, address: Bech32Address, tokens: Vec<TokenBalance>) {
        self.initial_balance.push((address, tokens));
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

    pub fn current_testing_sc_address(&self) -> &Bech32Address {
        self.testing_sc_address.as_ref().expect(NO_KNOWN_TESTING_SC)
    }

    pub fn current_chain_config_sc_address(&self) -> &Bech32Address {
        self.chain_config_sc_addresses
            .as_ref()
            .expect(NO_KNOWN_CHAIN_CONFIG_SC)
            .first()
    }

    pub fn current_sovereign_forge_sc_address(&self) -> &Bech32Address {
        self.sovereign_forge_sc_addresses
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

    pub fn current_enshrine_esdt_safe_address(&self) -> &Bech32Address {
        self.enshrine_esdt_safe_sc_addresses
            .as_ref()
            .expect(NO_KNOWN_ENSHRINE_ESDT_SAFE_SC)
            .first()
    }

    pub fn current_token_handler_address(&self) -> &Bech32Address {
        self.token_handler_addresses
            .as_ref()
            .expect(NO_KNOWN_TOKEN_HANDLER_SC)
            .first()
            .expect(NO_KNOWN_TOKEN_HANDLER_IN_THE_SPECIFIED_SHARD)
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

    pub fn get_nft_token_id_string(&self) -> String {
        self.nft_token_id
            .as_ref()
            .expect(NO_KNOWN_NFT_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_meta_esdt_token_id_string(&self) -> String {
        self.meta_esdt_token_id
            .as_ref()
            .expect(NO_KNOWN_META_ESDT_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_dynamic_nft_token_id_string(&self) -> String {
        self.dynamic_nft_token_id
            .as_ref()
            .expect(NO_KNOWN_DYNAMIC_NFT_TOKEN_ID)
            .token_id
            .clone()
    }

    pub fn get_sft_token_id_string(&self) -> String {
        self.sft_token_id
            .as_ref()
            .expect(NO_KNOWN_SFT_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_dynamic_sft_token_id_string(&self) -> String {
        self.dynamic_sft_token_id
            .as_ref()
            .expect(NO_KNOWN_DYNAMIC_SFT_TOKEN_ID)
            .token_id
            .clone()
    }

    pub fn get_dynamic_meta_esdt_token_id_string(&self) -> String {
        self.dynamic_meta_esdt_token_id
            .as_ref()
            .expect(NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID)
            .token_id
            .clone()
    }

    pub fn get_first_token_id(&self) -> TokenIdentifier<StaticApi> {
        self.first_token
            .as_ref()
            .expect(NO_KNOWN_FIRST_TOKEN)
            .token_id
            .as_str()
            .into()
    }

    pub fn get_first_token_id_as_esdt_info(&self) -> EsdtTokenInfo {
        self.first_token
            .as_ref()
            .expect(NO_KNOWN_FIRST_TOKEN)
            .clone()
    }

    pub fn get_fee_token_id(&self) -> TokenIdentifier<StaticApi> {
        self.fee_token
            .as_ref()
            .expect(NO_KNOWN_FEE_TOKEN)
            .token_id
            .as_str()
            .into()
    }

    pub fn get_second_token_id(&self) -> TokenIdentifier<StaticApi> {
        self.second_token
            .as_ref()
            .expect(NO_KNOWN_SECOND_TOKEN)
            .token_id
            .as_str()
            .into()
    }

    pub fn get_nft_token_id(&self) -> EsdtTokenInfo {
        self.nft_token_id
            .as_ref()
            .expect(NO_KNOWN_NFT_TOKEN)
            .clone()
    }

    pub fn get_meta_esdt_token_id(&self) -> EsdtTokenInfo {
        self.meta_esdt_token_id
            .as_ref()
            .expect(NO_KNOWN_META_ESDT_TOKEN)
            .clone()
    }

    pub fn get_dynamic_nft_token_id(&self) -> EsdtTokenInfo {
        self.dynamic_nft_token_id
            .as_ref()
            .expect(NO_KNOWN_DYNAMIC_NFT_TOKEN_ID)
            .clone()
    }

    pub fn get_sft_token_id(&self) -> EsdtTokenInfo {
        self.sft_token_id
            .as_ref()
            .expect(NO_KNOWN_SFT_TOKEN)
            .clone()
    }

    pub fn get_dynamic_sft_token_id(&self) -> EsdtTokenInfo {
        self.dynamic_sft_token_id
            .as_ref()
            .expect(NO_KNOWN_DYNAMIC_SFT_TOKEN_ID)
            .clone()
    }

    pub fn get_dynamic_meta_esdt_token_id(&self) -> EsdtTokenInfo {
        self.dynamic_meta_esdt_token_id
            .as_ref()
            .expect(NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID)
            .clone()
    }

    pub fn get_sov_to_mvx_token_id(&self) -> EsdtTokenInfo {
        self.sov_to_mvx_token_id
            .as_ref()
            .expect(NO_KNOWN_SOV_TO_MVX_TOKEN)
            .clone()
    }

    pub fn get_chain_factory_sc_address(&self, shard: u32) -> &Bech32Address {
        self.chain_factory_sc_addresses
            .as_ref()
            .expect(NO_KNOWN_CHAIN_FACTORY_SC)
            .get(shard as usize)
            .unwrap_or_else(|| panic!("No Chain Factory SC address for shard {}", shard))
    }

    pub fn get_token_handler_address(&self, shard: u32) -> &Bech32Address {
        self.token_handler_addresses
            .as_ref()
            .expect(NO_KNOWN_TOKEN_HANDLER_SC)
            .get(shard as usize)
            .unwrap_or_else(|| panic!("No Token Handler address for shard {}", shard))
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

    pub fn get_initial_balance_for_address(&self, address: Bech32Address) -> Vec<TokenBalance> {
        self.initial_balance
            .iter()
            .filter(|(addr, _)| *addr == address)
            .flat_map(|(_, balance)| balance.iter().cloned())
            .collect()
    }

    pub fn get_initial_token_balance_for_address(
        &self,
        address: Bech32Address,
        token_id: TokenIdentifier<StaticApi>,
    ) -> BigUint<StaticApi> {
        self.get_initial_balance_for_address(address)
            .into_iter()
            .find(|balance| balance.token_id == token_id.to_string())
            .map(|balance| balance.amount)
            .unwrap_or_else(|| BigUint::from(0u64))
    }
}
