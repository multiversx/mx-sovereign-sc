#![no_std]

use fee_common::storage;
use multiversx_sc::imports::*;
use structs::fee::FeeStruct;
pub mod fee_operations;
pub mod fee_whitelist;

#[multiversx_sc::contract]
pub trait SovFeeMarket:
    fee_whitelist::FeeWhitelistModule
    + storage::FeeCommonStorageModule
    + fee_operations::FeeOperationsModule
    + utils::UtilsModule
    + custom_events::CustomEventsModule
    + fee_common::endpoints::FeeCommonEndpointsModule
    + fee_common::helpers::FeeCommonHelpersModule
{
    #[init]
    fn init(&self, esdt_safe_address: ManagedAddress, fee: Option<FeeStruct<Self::Api>>) {
        self.init_fee_market(esdt_safe_address, fee);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
