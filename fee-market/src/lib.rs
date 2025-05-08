#![no_std]

use error_messages::ESDT_SAFE_ADDRESS_NOT_SET;
use fee_type::FeeStruct;

multiversx_sc::imports!();

pub mod fee_common;
pub mod fee_type;
pub mod price_aggregator;
pub mod subtract_fee;

#[multiversx_sc::contract]
pub trait FeeMarket:
    fee_common::CommonFeeModule
    + fee_type::FeeTypeModule
    + subtract_fee::SubtractFeeModule
    + price_aggregator::PriceAggregatorModule
    + utils::UtilsModule
    + setup_phase::SetupPhaseModule
{
    #[init]
    fn init(&self, esdt_safe_address: ManagedAddress, fee: Option<FeeStruct<Self::Api>>) {
        self.require_sc_address(&esdt_safe_address);
        self.esdt_safe_address().set(esdt_safe_address);

        match fee {
            Some(fee_struct) => self.set_fee(fee_struct),
            _ => self.fee_enabled().set(false),
        }
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[endpoint(setPriceAggregatorAddress)]
    fn set_price_aggregator_address(&self, price_aggregator_address: ManagedAddress) {
        self.require_sc_address(&price_aggregator_address);
        self.price_aggregator_address()
            .set(price_aggregator_address);
    }

    #[only_owner]
    fn complete_setup_phase(&self, header_verifier_address: ManagedAddress) {
        if self.is_setup_phase_complete() {
            return;
        }

        require!(
            !self.esdt_safe_address().is_empty(),
            ESDT_SAFE_ADDRESS_NOT_SET
        );

        self.tx()
            .to(ESDTSystemSCAddress)
            .typed(UserBuiltinProxy)
            .change_owner_address(&header_verifier_address)
            .sync_call();

        self.setup_phase_complete().set(true);
    }
}
