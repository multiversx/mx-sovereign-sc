#![no_std]

use operation::SovereignConfig;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CrossChainCommon {
    #[storage_mapper("sovereignConfig")]
    fn sovereign_config(&self) -> SingleValueMapper<SovereignConfig<Self::Api>>;
}
