#![no_std]

use operation::CrossChainConfig;

multiversx_sc::imports!();
#[multiversx_sc::module]
pub trait CrossChainCommon {
    #[storage_mapper("crossChainConfig")]
    fn cross_chain_config(&self) -> SingleValueMapper<CrossChainConfig<Self::Api>>;

    #[storage_mapper("feeMarketAddress")]
    fn fee_market_address(&self) -> SingleValueMapper<ManagedAddress>;
}
