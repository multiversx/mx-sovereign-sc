#![no_std]

use crate::err_msg;
use error_messages::{
    ADDRESS_NOT_VALID_SC_ADDRESS, CHAIN_FACTORY_ADDRESS_NOT_IN_EXPECTED_SHARD, DEPLOY_COST_IS_ZERO,
};
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
    + utils::UtilsModule
    + custom_events::CustomEventsModule
{
    #[init]
    fn init(&self, opt_deploy_cost: OptionalValue<BigUint>) {
        if let OptionalValue::Some(deploy_cost) = opt_deploy_cost {
            require!(deploy_cost > 0, DEPLOY_COST_IS_ZERO);
            self.deploy_cost().set(deploy_cost);
        }
    }

    #[only_owner]
    #[endpoint(registerChainFactory)]
    fn register_chain_factory(&self, shard_id: u32, chain_factory_address: ManagedAddress) {
        require!(
            self.blockchain()
                .get_shard_of_address(&chain_factory_address)
                == shard_id,
            CHAIN_FACTORY_ADDRESS_NOT_IN_EXPECTED_SHARD,
        );
        require!(
            self.blockchain().is_smart_contract(&chain_factory_address),
            ADDRESS_NOT_VALID_SC_ADDRESS
        );

        self.chain_factories(shard_id).set(chain_factory_address);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
