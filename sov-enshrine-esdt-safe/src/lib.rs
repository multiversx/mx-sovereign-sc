#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod deposit;

#[multiversx_sc::contract]
pub trait SovEnshrineEsdtSafe:
    cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + cross_chain::storage::CrossChainStorage
    + cross_chain::events::EventsModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[init]
    fn init(&self, token_handler_address: ManagedAddress) {
        self.require_sc_address(&token_handler_address);
        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
