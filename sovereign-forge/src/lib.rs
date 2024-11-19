#![no_std]

use crate::err_msg;
use multiversx_sc::imports::*;

mod common;
mod phases;

#[multiversx_sc::contract]
pub trait SovereignForge:
    phases::PhasesModule + common::storage::StorageModule + common::utils::UtilsModule
{
    #[init]
    fn init(&self, deploy_cost: BigUint) {
        require!(deploy_cost > 0, "The deploy cost can't be a 0 value");
        self.deploy_cost().set(deploy_cost);
    }

    #[endpoint(registerTokenHandler)]
    fn register_token_handler(&self, token_handler_address: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&token_handler_address),
            "The given address is not a valid SC address"
        );

        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();
        let caller_shard_id = blockchain_api.get_shard_of_address(&caller);

        self.token_handlers(caller_shard_id)
            .set(token_handler_address);
    }

    #[endpoint(registerChainFactory)]
    fn register_chain_factory(&self, chain_factory_address: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&chain_factory_address),
            "The given address is not a valid SC address"
        );

        let caller = self.blockchain().get_caller();
        let caller_shard_id = self.blockchain().get_shard_of_address(&caller);

        self.token_handlers(caller_shard_id)
            .set(chain_factory_address);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
