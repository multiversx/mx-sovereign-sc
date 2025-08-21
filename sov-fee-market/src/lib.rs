#![no_std]

use fee_common::storage;
#[allow(unused_imports)]
use multiversx_sc::imports::*;
pub mod fee_operations;
pub mod fee_whitelist;

#[multiversx_sc::contract]
pub trait SovFeeMarket:
    fee_whitelist::FeeWhitelistModule + storage::FeeCommonStorageModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
