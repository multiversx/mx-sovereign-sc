#![no_std]

multiversx_sc::imports!();

pub mod config_operations;
pub mod fee_operations;

#[multiversx_sc::contract]
pub trait SovRegistrar:
    fee_operations::FeeOperationsModule
    + custom_events::CustomEventsModule
    + tx_nonce::TxNonceModule
    + fee_common::helpers::FeeCommonHelpersModule
    + fee_common::storage::FeeCommonStorageModule
    + utils::UtilsModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
