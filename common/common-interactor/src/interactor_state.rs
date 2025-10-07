#![allow(non_snake_case)]

use error_messages::{
    NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID, NO_KNOWN_DYNAMIC_NFT_TOKEN_ID,
    NO_KNOWN_DYNAMIC_SFT_TOKEN_ID, NO_KNOWN_FEE_TOKEN, NO_KNOWN_FIRST_TOKEN,
    NO_KNOWN_FUNGIBLE_TOKEN, NO_KNOWN_META_ESDT_TOKEN, NO_KNOWN_NFT_TOKEN, NO_KNOWN_SFT_TOKEN,
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
    pub fungible_tokens: Vec<EsdtTokenInfo>,
    pub fee_token: Option<EsdtTokenInfo>,
    pub nft_tokens: Vec<EsdtTokenInfo>,
    pub meta_esdt_tokens: Vec<EsdtTokenInfo>,
    pub dynamic_nft_tokens: Vec<EsdtTokenInfo>,
    pub dynamic_sft_tokens: Vec<EsdtTokenInfo>,
    pub dynamic_meta_esdt_tokens: Vec<EsdtTokenInfo>,
    pub sft_tokens: Vec<EsdtTokenInfo>,
    pub sov_to_mvx_token_id: Option<EsdtTokenInfo>,
    pub initial_wallet_tokens_state: Vec<EsdtTokenInfo>,
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

    // Legacy methods for backward compatibility
    pub fn set_nft_token_id(&mut self, token: EsdtTokenInfo) {
        self.nft_tokens.clear();
        self.nft_tokens.push(token);
    }

    pub fn set_meta_esdt_token_id(&mut self, token: EsdtTokenInfo) {
        self.meta_esdt_tokens.clear();
        self.meta_esdt_tokens.push(token);
    }

    pub fn set_dynamic_nft_token_id(&mut self, token: EsdtTokenInfo) {
        self.dynamic_nft_tokens.clear();
        self.dynamic_nft_tokens.push(token);
    }

    pub fn set_sft_token_id(&mut self, token: EsdtTokenInfo) {
        self.sft_tokens.clear();
        self.sft_tokens.push(token);
    }

    pub fn set_dynamic_sft_token_id(&mut self, token: EsdtTokenInfo) {
        self.dynamic_sft_tokens.clear();
        self.dynamic_sft_tokens.push(token);
    }

    pub fn set_dynamic_meta_esdt_token_id(&mut self, token: EsdtTokenInfo) {
        self.dynamic_meta_esdt_tokens.clear();
        self.dynamic_meta_esdt_tokens.push(token);
    }

    pub fn set_sov_to_mvx_token_id(&mut self, token: EsdtTokenInfo) {
        self.sov_to_mvx_token_id = Some(token);
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

    pub fn get_nft_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.nft_tokens
            .first()
            .expect(NO_KNOWN_NFT_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_meta_esdt_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.meta_esdt_tokens
            .first()
            .expect(NO_KNOWN_META_ESDT_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_dynamic_nft_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.dynamic_nft_tokens
            .first()
            .expect(NO_KNOWN_DYNAMIC_NFT_TOKEN_ID)
            .token_id
            .clone()
    }

    pub fn get_sft_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.sft_tokens
            .first()
            .expect(NO_KNOWN_SFT_TOKEN)
            .token_id
            .clone()
    }

    pub fn get_dynamic_sft_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.dynamic_sft_tokens
            .first()
            .expect(NO_KNOWN_DYNAMIC_SFT_TOKEN_ID)
            .token_id
            .clone()
    }

    pub fn get_dynamic_meta_esdt_token_identifier(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.dynamic_meta_esdt_tokens
            .first()
            .expect(NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID)
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

    pub fn get_nft_token_id(&self) -> EsdtTokenInfo {
        self.nft_tokens.first().expect(NO_KNOWN_NFT_TOKEN).clone()
    }

    pub fn get_meta_esdt_token_id(&self) -> EsdtTokenInfo {
        self.meta_esdt_tokens
            .first()
            .expect(NO_KNOWN_META_ESDT_TOKEN)
            .clone()
    }

    pub fn get_dynamic_nft_token_id(&self) -> EsdtTokenInfo {
        self.dynamic_nft_tokens
            .first()
            .expect(NO_KNOWN_DYNAMIC_NFT_TOKEN_ID)
            .clone()
    }

    pub fn get_sft_token_id(&self) -> EsdtTokenInfo {
        self.sft_tokens.first().expect(NO_KNOWN_SFT_TOKEN).clone()
    }

    pub fn get_dynamic_sft_token_id(&self) -> EsdtTokenInfo {
        self.dynamic_sft_tokens
            .first()
            .expect(NO_KNOWN_DYNAMIC_SFT_TOKEN_ID)
            .clone()
    }

    pub fn get_dynamic_meta_esdt_token_id(&self) -> EsdtTokenInfo {
        self.dynamic_meta_esdt_tokens
            .first()
            .expect(NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID)
            .clone()
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

    pub fn get_sov_to_mvx_token_id(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        self.sov_to_mvx_token_id
            .as_ref()
            .expect(NO_KNOWN_SOV_TO_MVX_TOKEN)
            .token_id
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
}
