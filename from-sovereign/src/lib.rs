#![no_std]

use multiversx_sc::imports::*;
use operation::CrossChainConfig;

pub mod deposit;
pub mod token_mapping;

#[multiversx_sc::contract]
pub trait FromSovereign:
    deposit::DepositModule
    + token_mapping::TokenMappingModule
    + cross_chain::CrossChainCommon
    + multiversx_sc_modules::pause::PauseModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + utils::UtilsModule
    + cross_chain::events::EventsModule
    + cross_chain::storage::CrossChainStorage
{
    #[init]
    fn init(&self, cross_chain_config: CrossChainConfig<Self::Api>) {
        self.cross_chain_config().set(cross_chain_config);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
