#![no_std]

use error_messages::{ERROR_AT_ENCODING, INVALID_REGISTRATION_STATUS};
use multiversx_sc::imports::*;
use structs::{configs::SovereignConfig, generate_hash::GenerateHash};

multiversx_sc::imports!();

pub mod validator_rules;

#[multiversx_sc::contract]
pub trait ChainConfigContract:
    validator_rules::ValidatorRulesModule
    + setup_phase::SetupPhaseModule
    + utils::UtilsModule
    + events::EventsModule
{
    #[init]
    fn init(&self, opt_config: OptionalValue<SovereignConfig<Self::Api>>) {
        let new_config = match opt_config {
            OptionalValue::Some(cfg) => {
                if let Some(error_message) = self.is_new_config_valid(&cfg) {
                    sc_panic!(error_message);
                }
                cfg
            }
            OptionalValue::None => SovereignConfig::default_config(),
        };

        self.sovereign_config().set(new_config.clone());
        self.genesis_phase_status().set(true);
    }

    #[only_owner]
    #[endpoint(updateSovereignConfigSetupPhase)]
    fn update_sovereign_config_during_setup_phase(&self, new_config: SovereignConfig<Self::Api>) {
        if let Some(error_message) = self.is_new_config_valid(&new_config) {
            sc_panic!(error_message);
        }
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
        if config_hash.is_empty() {
            self.failed_bridge_operation_event(
                &hash_of_hashes,
                &config_hash,
                &ManagedBuffer::from(ERROR_AT_ENCODING),
            );

            self.remove_executed_hash(&hash_of_hashes, &config_hash);
            return;
        };

        self.lock_operation_hash(&config_hash, &hash_of_hashes);

        if let Some(error_message) = self.is_new_config_valid(&new_config) {
            self.failed_bridge_operation_event(
                &hash_of_hashes,
                &config_hash,
                &ManagedBuffer::from(error_message),
            );

            self.remove_executed_hash(&hash_of_hashes, &config_hash);
            return;
        } else {
            self.sovereign_config().set(new_config);
        }

        self.remove_executed_hash(&hash_of_hashes, &config_hash);
        self.execute_bridge_operation_event(&hash_of_hashes, &config_hash);
    }

    #[endpoint(resumeRegistration)]
    fn update_registration_status(&self, hash_of_hashes: ManagedBuffer, registration_status: u8) {
        self.require_setup_complete();

        let status_hash = ManagedBuffer::new_from_bytes(
            &self
                .crypto()
                .sha256(ManagedBuffer::new_from_bytes(&[registration_status]))
                .to_byte_array(),
        );

        if registration_status != 0u8 || registration_status != 1u8 {
            self.failed_bridge_operation_event(
                &hash_of_hashes,
                &status_hash,
                &ManagedBuffer::from(INVALID_REGISTRATION_STATUS),
            );

            self.remove_executed_hash(&hash_of_hashes, &status_hash);

            return;
        }

        self.lock_operation_hash(&status_hash, &hash_of_hashes);

        self.registration_status().set(1);

        self.remove_executed_hash(&hash_of_hashes, &status_hash);
        self.registration_status_update_event();
    }

    // #[only_owner]
    // #[endpoint(freezeRegistration)]
    // fn freeze_registration

    #[only_owner]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self) {
        if self.is_setup_phase_complete() {
            return;
        }
        self.require_validator_set_valid(self.bls_keys_map().len());

        self.genesis_phase_status().set(false);
        self.registration_status().set(0);
        self.complete_genesis_event();
        self.setup_phase_complete().set(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
