#![no_std]

use multiversx_sc::imports::*;
use operation::CrossChainConfig;

pub mod deposit;

#[multiversx_sc::contract]
pub trait FromSovereign:
    deposit::DepositModule
    + cross_chain::CrossChainCommon
    + multiversx_sc_modules::pause::PauseModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + utils::UtilsModule
    + cross_chain::events::EventsModule
{
    #[init]
    fn init(&self, cross_chain_config: CrossChainConfig<Self::Api>) {
        self.cross_chain_config().set(cross_chain_config);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
