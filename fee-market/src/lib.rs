#![no_std]

use fee_type::FeeStruct;

multiversx_sc::imports!();

pub mod fee_common;
pub mod fee_market_proxy;
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
    + bls_signature::BlsSignatureModule
{
    #[init]
    fn init(
        &self,
        esdt_safe_address: ManagedAddress,
        price_aggregator_address: ManagedAddress,
        fee: Option<FeeStruct<Self::Api>>,
    ) {
        self.require_sc_address(&esdt_safe_address);
        self.require_sc_address(&price_aggregator_address);

        self.esdt_safe_address().set(esdt_safe_address);
        self.price_aggregator_address()
            .set(price_aggregator_address);

        match fee {
            Option::Some(fee_struct) => self.set_fee(fee_struct.base_token, fee_struct.fee_type),
            _ => self.fee_enabled().set(false),
        }
    }

    #[upgrade]
    fn upgrade(&self) {}
}
