#![allow(non_snake_case)]

use error_messages::{
    NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID, NO_KNOWN_DYNAMIC_NFT_TOKEN_ID,
    NO_KNOWN_DYNAMIC_SFT_TOKEN_ID, NO_KNOWN_FEE_TOKEN, NO_KNOWN_FIRST_TOKEN,
    NO_KNOWN_META_ESDT_TOKEN, NO_KNOWN_NFT_TOKEN, NO_KNOWN_SECOND_TOKEN, NO_KNOWN_SFT_TOKEN,
    NO_KNOWN_SOV_TO_MVX_TOKEN,
};
use multiversx_sc_snippets::imports::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct EsdtTokenInfo {
    pub token_id: EgldOrEsdtTokenIdentifier<StaticApi>,
    pub nonce: u64,
    pub token_type: EsdtTokenType,
    pub decimals: usize,
    pub amount: BigUint<StaticApi>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: Bech32Address,
    pub chain_id: String,
}

// NOTE: This struct holds deployed contract addresses.
// The index of each address corresponds to the shard number where the contract was deployed.
// For example, index 0 = shard 0, index 1 = shard 1, etc.
#[derive(Debug, Default, Serialize, Deserialize)]
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
    pub initial_wallet_tokens_state: Option<Vec<EsdtTokenInfo>>,
    pub sovereign_owners: Option<Vec<Address>>,
    pub bridge_owners: Option<Vec<Address>>,
    pub bridge_services: Option<Vec<Address>>,
}

impl State {
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

    pub fn set_initial_wallet_tokens_state(&mut self, tokens: Vec<EsdtTokenInfo>) {
        self.initial_wallet_tokens_state = Some(tokens);
    }

    pub fn set_bridge_owners(&mut self, owners: Vec<Address>) {
        self.bridge_owners = Some(owners);
    }

    pub fn set_bridge_services(&mut self, services: Vec<Address>) {
        self.bridge_services = Some(services);
    }

    pub fn set_sovereign_owners(&mut self, owners: Vec<Address>) {
        self.sovereign_owners = Some(owners);
    }

    pub fn get_bridge_owners(&self) -> Vec<Address> {
        self.bridge_owners.clone().unwrap_or_default()
    }

    pub fn get_bridge_services(&self) -> Vec<Address> {
        self.bridge_services.clone().unwrap_or_default()
    }

    pub fn get_sovereign_owners(&self) -> Vec<Address> {
        self.sovereign_owners.clone().unwrap_or_default()
    }

    pub fn get_first_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.first_token
            .as_ref()
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

    pub fn get_second_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.second_token
            .as_ref()
            .expect(NO_KNOWN_SECOND_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_nft_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.nft_token_id
            .as_ref()
            .expect(NO_KNOWN_NFT_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_meta_esdt_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.meta_esdt_token_id
            .as_ref()
            .expect(NO_KNOWN_META_ESDT_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_dynamic_nft_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.dynamic_nft_token_id
            .as_ref()
            .expect(NO_KNOWN_DYNAMIC_NFT_TOKEN_ID)
            .token_id
            .clone()
    }

    pub fn get_sft_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.sft_token_id
            .as_ref()
            .expect(NO_KNOWN_SFT_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_dynamic_sft_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.dynamic_sft_token_id
            .as_ref()
            .expect(NO_KNOWN_DYNAMIC_SFT_TOKEN_ID)
            .token_id
            .clone()
    }

    pub fn get_dynamic_meta_esdt_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.dynamic_meta_esdt_token_id
            .as_ref()
            .expect(NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID)
            .token_id
            .clone()
    }

    pub fn get_first_token_id(&self) -> EsdtTokenInfo {
        self.first_token
            .as_ref()
            .expect(NO_KNOWN_FIRST_TOKEN)
            .clone()
    }

    pub fn get_fee_token_id(&self) -> EsdtTokenInfo {
        self.fee_token.as_ref().expect(NO_KNOWN_FEE_TOKEN).clone()
    }

    pub fn get_second_token_id(&self) -> EsdtTokenInfo {
        self.second_token
            .as_ref()
            .expect(NO_KNOWN_SECOND_TOKEN)
            .clone()
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

    pub fn get_sov_to_mvx_token_id(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.sov_to_mvx_token_id
            .as_ref()
            .expect(NO_KNOWN_SOV_TO_MVX_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_initial_wallet_tokens_state(&self) -> &Option<Vec<EsdtTokenInfo>> {
        &self.initial_wallet_tokens_state
    }

    pub fn get_initial_wallet_token_balance(
        &self,
        token_id: EgldOrEsdtTokenIdentifier<StaticApi>,
    ) -> BigUint<StaticApi> {
        self.initial_wallet_tokens_state
            .as_ref()
            .expect("No initial wallet tokens state set")
            .iter()
            .find(|token| token.token_id == token_id)
            .map_or_else(BigUint::zero, |token| token.amount.clone())
    }
}
