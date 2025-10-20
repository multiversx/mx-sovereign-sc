#![no_std]

use crate::err_msg;
use error_messages::{
    ADDRESS_NOT_VALID_SC_ADDRESS, CHAIN_FACTORY_ADDRESS_NOT_IN_EXPECTED_SHARD, DEPLOY_COST_IS_ZERO,
};
use multiversx_sc::imports::*;

pub mod forge_common;
pub mod phases;
pub mod update_configs;

#[multiversx_sc::contract]
pub trait SovereignForge:
    phases::PhasesModule
    + forge_common::storage::StorageModule
    + forge_common::forge_utils::ForgeUtilsModule
    + forge_common::sc_deploy::ScDeployModule
    + forge_common::callbacks::ForgeCallbackModule
    + update_configs::UpdateConfigsModule
    + common_utils::CommonUtilsModule
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
            shard_id < forge_common::forge_utils::NUMBER_OF_SHARDS,
            "Shard id {} is out of range",
            shard_id
        );
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

    #[only_owner]
    #[endpoint(registerTrustedToken)]
    fn register_trusted_token(&self, trusted_token: ManagedBuffer) {
        self.trusted_tokens().insert(trusted_token);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
