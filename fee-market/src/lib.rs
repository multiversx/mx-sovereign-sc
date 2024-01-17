#![no_std]

multiversx_sc::imports!();

pub mod enable_fee;
pub mod fee_common;
pub mod fee_type;
pub mod pairs;
pub mod subtract_fee;

#[multiversx_sc::contract]
pub trait FeeMarket:
    enable_fee::EnableFeeModule
    + fee_common::CommonFeeModule
    + fee_type::FeeTypeModule
    + subtract_fee::SubtractFeeModule
    + pairs::PairsModule
    + utils::UtilsModule
{
    #[init]
    fn init(&self, esdt_safe_address: ManagedAddress, pair_for_query: ManagedAddress) {
        self.require_sc_address(&esdt_safe_address);
        self.require_sc_address(&pair_for_query);

        self.esdt_safe_address().set(esdt_safe_address);
        self.pair_for_query().set(pair_for_query);
        self.fee_enabled().set(true);
    }

    #[endpoint]
    fn upgrade(&self) {}
}
