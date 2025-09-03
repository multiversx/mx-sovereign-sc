#![no_std]

multiversx_sc::imports!();

pub mod config_operations;
pub mod fee_operations;

#[multiversx_sc::contract]
pub trait SovRegistrar:
    fee_operations::FeeOperationsModule
    + config_operations::ConfigOperationsModule
    + custom_events::CustomEventsModule
    + tx_nonce::TxNonceModule
    + fee_common::helpers::FeeCommonHelpersModule
    + fee_common::storage::FeeCommonStorageModule
    + common_utils::CommonUtilsModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
