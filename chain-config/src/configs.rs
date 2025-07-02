use error_messages::{ERROR_AT_ENCODING, INVALID_REGISTRATION_STATUS};
use structs::{configs::SovereignConfig, generate_hash::GenerateHash};

use crate::{
    config_utils::{self, DISABLED, ENABLED},
    storage, validator,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ConfigsModule:
    validator::ValidatorModule
    + storage::ChainConfigStorageModule
    + utils::UtilsModule
    + config_utils::ChainConfigUtilsModule
    + events::EventsModule
    + setup_phase::SetupPhaseModule
{
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

    #[endpoint(updateRegistrationStatus)]
    fn update_registration_status(&self, hash_of_hashes: ManagedBuffer, registration_status: u8) {
        self.require_setup_complete();

        let status_hash = ManagedBuffer::new_from_bytes(
            &self
                .crypto()
                .sha256(ManagedBuffer::new_from_bytes(&[registration_status]))
                .to_byte_array(),
        );

        self.lock_operation_hash(&status_hash, &hash_of_hashes);

        if registration_status != DISABLED && registration_status != ENABLED {
            self.failed_bridge_operation_event(
                &hash_of_hashes,
                &status_hash,
                &ManagedBuffer::from(INVALID_REGISTRATION_STATUS),
            );

            self.remove_executed_hash(&hash_of_hashes, &status_hash);

            return;
        }

        let registration_status_mapper = self.registration_status();

        registration_status_mapper.set(registration_status);

        self.remove_executed_hash(&hash_of_hashes, &status_hash);
        self.registration_status_update_event(&self.get_event_msg(registration_status));
    }
}
