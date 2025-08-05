use multiversx_sc::{
    imports::Bech32Address,
    types::{BigUint, EsdtTokenType},
};
use multiversx_sc_snippets::imports::StaticApi;
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

#[derive(Clone)]
pub struct ActionConfig<'a> {
    pub shard: u32,
    pub expected_error: Option<&'a str>,
    pub expected_log: Option<&'a str>,
    pub expected_log_error: Option<&'a str>,
    pub is_sovereign: bool,
    pub with_transfer_data: Option<bool>,
    pub decimals: Option<usize>,
    pub token_type: Option<EsdtTokenType>,
    pub nonce: Option<u64>,
    pub endpoint: Option<&'a str>,
}

impl<'a> ActionConfig<'a> {
    pub fn new(shard: u32) -> Self {
        Self {
            shard,
            expected_error: None,
            expected_log: None,
            expected_log_error: None,
            is_sovereign: false,
            with_transfer_data: None,
            decimals: None,
            token_type: None,
            nonce: None,
            endpoint: None,
        }
    }

    pub fn expect_error(mut self, error: &'a str) -> Self {
        self.expected_error = Some(error);
        self
    }

    pub fn expect_log(mut self, log: &'a str) -> Self {
        self.expected_log = Some(log);
        self
    }

    pub fn sovereign(mut self) -> Self {
        self.is_sovereign = true;
        self
    }

    pub fn with_transfer_data(mut self) -> Self {
        self.with_transfer_data = Some(true);
        self
    }

    pub fn for_register(mut self, token_type: EsdtTokenType, decimals: usize, nonce: u64) -> Self {
        self.token_type = Some(token_type);
        self.decimals = Some(decimals);
        self.nonce = Some(nonce);
        self
    }

    pub fn with_endpoint(mut self, endpoint: &'a str) -> Self {
        self.endpoint = Some(endpoint);
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
    pub is_sovereign_token: bool,
    pub is_execute: bool,
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

    pub fn amount(mut self, amount: Option<BigUint<StaticApi>>) -> Self {
        self.amount = amount;
        self
    }

    pub fn fee(mut self, fee: Option<FeeStruct<StaticApi>>) -> Self {
        self.fee = fee;
        self
    }

    pub fn with_transfer_data(mut self, value: bool) -> Self {
        self.with_transfer_data = value;
        self
    }

    pub fn is_sovereign_token(mut self, value: bool) -> Self {
        self.is_sovereign_token = value;
        self
    }

    pub fn is_execute(mut self, value: bool) -> Self {
        self.is_execute = value;
        self
    }
}

pub enum EsdtSafeType {
    MvxEsdtSafe,
    EnshrineEsdtSafe,
}

#[derive(Clone)]
pub struct TemplateAddresses {
    pub chain_config_address: Bech32Address,
    pub header_verifier_address: Bech32Address,
    pub esdt_safe_address: Bech32Address,
    pub fee_market_address: Bech32Address,
}
