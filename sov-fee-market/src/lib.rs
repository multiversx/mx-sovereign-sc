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
        self.require_sc_address(&esdt_safe_address);
        self.esdt_safe_address().set(esdt_safe_address);

        match fee {
            Some(fee_struct) => {
                let _ = self.set_fee_in_storage(&fee_struct);
            }
            _ => self.fee_enabled().set(false),
        }
    }

    #[upgrade]
    fn upgrade(&self) {}
}
