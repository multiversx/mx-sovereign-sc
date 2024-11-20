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

    #[only_owner]
    #[endpoint(registerTokenHandler)]
    fn register_token_handler(&self, shard_id: u32, token_handler_address: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&token_handler_address),
            "The given address is not a valid SC address"
        );

        self.token_handlers(shard_id).set(token_handler_address);
        self.is_setup_complte(shard_id).set(false);
    }

    #[only_owner]
    #[endpoint(registerChainFactory)]
    fn register_chain_factory(&self, shard_id: u32, chain_factory_address: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&chain_factory_address),
            "The given address is not a valid SC address"
        );

        self.token_handlers(shard_id).set(chain_factory_address);
        self.is_setup_complte(shard_id).set(false);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
