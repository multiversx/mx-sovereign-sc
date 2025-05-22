#![no_std]

use cross_chain::{execute_common, storage};
use structs::{configs::SovereignConfig, generate_hash::GenerateHash};

multiversx_sc::imports!();

pub mod validator_rules;

#[multiversx_sc::contract]
pub trait ChainConfigContract:
    validator_rules::ValidatorRulesModule
    + setup_phase::SetupPhaseModule
    + execute_common::ExecuteCommonModule
    + storage::CrossChainStorage
{
    #[init]
    fn init(&self, config: SovereignConfig<Self::Api>) {
        self.require_valid_config(&config);
        self.sovereign_config().set(config.clone());
    }

    #[only_owner]
    #[endpoint(updateSovereignConfig)]
    fn update_sovereign_config(
        &self,
        hash_of_hashes: ManagedBuffer,
        new_config: SovereignConfig<Self::Api>,
    ) {
        let opt_hash = if self.is_setup_phase_complete() {
            Some(new_config.generate_hash())
        } else {
            None
        };

        if let Some(ref config_hash) = opt_hash {
            self.lock_operation_hash(config_hash, &hash_of_hashes);
        }

        self.require_valid_config(&new_config);
        self.sovereign_config().set(new_config);

        if let Some(config_hash) = opt_hash {
            self.remove_executed_hash(&hash_of_hashes, &config_hash);
        }
    }

    #[only_owner]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self) {
        if self.is_setup_phase_complete() {
            return;
        }

        self.setup_phase_complete().set(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
