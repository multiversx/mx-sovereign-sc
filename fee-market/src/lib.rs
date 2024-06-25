#![no_std]

multiversx_sc::imports!();

pub mod enable_fee;
pub mod fee_common;
pub mod fee_market_proxy;
pub mod fee_type;
pub mod price_aggregator;
pub mod safe_price_query;
pub mod subtract_fee;

#[multiversx_sc::contract]
pub trait FeeMarket:
    enable_fee::EnableFeeModule
    + fee_common::CommonFeeModule
    + fee_type::FeeTypeModule
    + subtract_fee::SubtractFeeModule
    + price_aggregator::PriceAggregatorModule
    + safe_price_query::SafePriceQueryModule
    + utils::UtilsModule
    + bls_signature::BlsSignatureModule
{
    #[init]
    fn init(
        &self,
        esdt_safe_address: ManagedAddress,
        price_aggregator_address: ManagedAddress,
        usdc_token_id: TokenIdentifier,
        wegld_token_id: TokenIdentifier,
    ) {
        self.require_sc_address(&esdt_safe_address);
        self.require_sc_address(&price_aggregator_address);

        self.esdt_safe_address().set(esdt_safe_address);
        self.price_aggregator_address()
            .set(price_aggregator_address);
        self.usdc_token_id().set(usdc_token_id);
        self.wegld_token_id().set(wegld_token_id);
        self.fee_enabled().set(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
