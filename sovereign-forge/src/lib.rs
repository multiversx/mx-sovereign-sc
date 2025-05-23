#![no_std]

use crate::err_msg;
use error_messages::{ADDRESS_NOT_VALID_SC_ADDRESS, DEPLOY_COST_IS_ZERO};
use multiversx_sc::imports::*;

pub mod common;
pub mod phases;
pub mod update_configs;

#[multiversx_sc::contract]
pub trait SovereignForge:
    phases::PhasesModule
    + common::storage::StorageModule
    + common::utils::UtilsModule
    + common::sc_deploy::ScDeployModule
    + update_configs::UpdateConfigsModule
{
    #[init]
    fn init(&self, deploy_cost: BigUint) {
        require!(deploy_cost > 0, DEPLOY_COST_IS_ZERO);
        self.deploy_cost().set(deploy_cost);
    }

    #[only_owner]
    #[endpoint(registerTokenHandler)]
    fn register_token_handler(&self, shard_id: u32, token_handler_address: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&token_handler_address),
            ADDRESS_NOT_VALID_SC_ADDRESS
        );

        self.token_handlers(shard_id).set(token_handler_address);
    }

    #[only_owner]
    #[endpoint(registerChainFactory)]
    fn register_chain_factory(&self, shard_id: u32, chain_factory_address: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&chain_factory_address),
            ADDRESS_NOT_VALID_SC_ADDRESS
        );

        self.chain_factories(shard_id).set(chain_factory_address);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
