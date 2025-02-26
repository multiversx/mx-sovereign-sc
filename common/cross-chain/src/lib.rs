#![no_std]

pub mod deposit_common;
pub mod deposit_unit_tests_setup;
pub mod events;
pub mod execute_common;
pub mod storage;

pub const MAX_TRANSFERS_PER_TX: usize = 10;
pub const DEFAULT_ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD
pub const REGISTER_GAS: u64 = 60_000_000;
