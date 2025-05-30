#![no_std]

use error_messages::MAX_GAS_LIMIT_PER_TX_EXCEEDED;
use structs::configs::EsdtSafeConfig;
multiversx_sc::imports!();

pub mod deposit_common;
pub mod execute_common;
pub mod storage;

pub const MAX_TRANSFERS_PER_TX: usize = 10;
pub const DEFAULT_ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD
pub const REGISTER_GAS: u64 = 60_000_000;
pub const MAX_GAS_PER_TRANSACTION: u64 = 600_000_000;

#[multiversx_sc::module]
pub trait LibCommon: crate::storage::CrossChainStorage {
    fn is_esdt_safe_config_valid(&self, config: &EsdtSafeConfig<Self::Api>) -> Option<&str> {
        if config.max_tx_gas_limit < MAX_GAS_PER_TRANSACTION {
            None
        } else {
            Some(MAX_GAS_LIMIT_PER_TX_EXCEEDED)
        }
    }
}
