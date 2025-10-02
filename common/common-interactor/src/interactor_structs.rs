use multiversx_sc::{
    imports::Bech32Address,
    types::{BigUint, EsdtTokenType},
};
use multiversx_sc_snippets::imports::StaticApi;
use serde::{Deserialize, Serialize};
use structs::fee::FeeStruct;

use crate::interactor_state::EsdtTokenInfo;

pub struct IssueTokenStruct {
    pub token_display_name: String,
    pub token_ticker: String,
    pub token_type: EsdtTokenType,
    pub num_decimals: usize,
}
#[derive(Clone)]
pub struct MintTokenStruct {
    pub name: Option<String>,
    pub amount: BigUint<StaticApi>,
    pub attributes: Option<Vec<u8>>,
}

#[derive(Clone, Default)]
pub struct ActionConfig {
    pub shard: u32,
    pub expected_error: Option<String>,
    pub expected_log: Option<Vec<String>>,
    pub expected_log_error: Option<String>,
    pub with_transfer_data: Option<bool>,
    pub endpoint: Option<String>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct SerializableFeeMarketToken {
    pub token_id: String,
    pub nonce: u64,
    pub token_type: u8,
    pub decimals: usize,
    pub amount: u64,
}

impl ActionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn shard(mut self, shard: u32) -> Self {
        self.shard = shard;
        self
    }

    pub fn expect_error(mut self, error: String) -> Self {
        self.expected_error = Some(error);
        self
    }

    pub fn expect_log(mut self, log: Vec<String>) -> Self {
        self.expected_log = Some(log);
        self
    }

    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self.with_transfer_data = Some(true);
        self
    }

    pub fn expected_log_error(mut self, value: String) -> Self {
        self.expected_log_error = Some(value);
        self
    }
}

#[derive(Clone, Default)]
pub struct BalanceCheckConfig {
    pub shard: u32,
    pub token: Option<EsdtTokenInfo>,
    pub amount: Option<BigUint<StaticApi>>,
    pub fee: Option<FeeStruct<StaticApi>>,
    pub with_transfer_data: bool,
    pub is_execute: bool,
    pub expected_error: Option<String>,
}

impl BalanceCheckConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn shard(mut self, shard: u32) -> Self {
        self.shard = shard;
        self
    }

    pub fn token(mut self, token: Option<EsdtTokenInfo>) -> Self {
        self.token = token;
        self
    }

    pub fn amount(mut self, amount: BigUint<StaticApi>) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn fee(mut self, fee: FeeStruct<StaticApi>) -> Self {
        self.fee = Some(fee);
        self
    }

    pub fn with_transfer_data(mut self, value: bool) -> Self {
        self.with_transfer_data = value;
        self
    }

    pub fn is_execute(mut self, value: bool) -> Self {
        self.is_execute = value;
        self
    }

    pub fn expected_error(mut self, value: Option<String>) -> Self {
        self.expected_error = value;
        self
    }
}

#[derive(Clone)]
pub struct TemplateAddresses {
    pub chain_config_address: Bech32Address,
    pub header_verifier_address: Bech32Address,
    pub esdt_safe_address: Bech32Address,
    pub fee_market_address: Bech32Address,
}
