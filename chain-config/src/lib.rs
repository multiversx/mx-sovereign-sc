#![no_std]

use cross_chain::events;
use structs::{configs::SovereignConfig, generate_hash::GenerateHash};

multiversx_sc::imports!();

pub mod validator_rules;

#[multiversx_sc::contract]
pub trait ChainConfigContract:
    validator_rules::ValidatorRulesModule + setup_phase::SetupPhaseModule + events::EventsModule
{
    #[init]
    fn init(&self, config: SovereignConfig<Self::Api>) {
        self.require_valid_config(&config);
        self.sovereign_config().set(config.clone());
    }

    #[only_owner]
    #[endpoint(updateSovereignConfigSetupPhase)]
    fn update_sovereign_config_during_setup_phase(&self, new_config: SovereignConfig<Self::Api>) {
        self.require_valid_config(&new_config);
        self.sovereign_config().set(new_config);
    }

    #[endpoint(updateSovereignConfig)]
    fn update_sovereign_config(
        &self,
        hash_of_hashes: ManagedBuffer,
        new_config: SovereignConfig<Self::Api>,
    ) {
        self.require_setup_complete();

        let config_hash = new_config.generate_hash();
        self.lock_operation_hash(&config_hash, &hash_of_hashes);

        if let Some(error_message) = self.is_new_config_valid(&new_config) {
            self.failed_bridge_operation_event(
                &hash_of_hashes,
                &config_hash,
                &ManagedBuffer::from(error_message),
            );
        } else {
            self.sovereign_config().set(new_config);
        }

        self.remove_executed_hash(&hash_of_hashes, &config_hash);
        self.execute_bridge_operation_event(&hash_of_hashes, &config_hash);
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
