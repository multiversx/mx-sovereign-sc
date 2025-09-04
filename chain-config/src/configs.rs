use error_messages::{ERROR_AT_ENCODING, SETUP_PHASE_ALREADY_COMPLETED, SETUP_PHASE_NOT_COMPLETED};
use structs::{configs::SovereignConfig, generate_hash::GenerateHash};

use crate::{config_utils, storage, validator};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ConfigsModule:
    validator::ValidatorModule
    + storage::ChainConfigStorageModule
    + utils::UtilsModule
    + config_utils::ChainConfigUtilsModule
    + custom_events::CustomEventsModule
    + setup_phase::SetupPhaseModule
{
    #[only_owner]
    #[endpoint(updateSovereignConfigSetupPhase)]
    fn update_sovereign_config_during_setup_phase(&self, new_config: SovereignConfig<Self::Api>) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

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
        let config_hash = new_config.generate_hash();
        if config_hash.is_empty() {
            self.complete_operation(
                &hash_of_hashes,
                &config_hash,
                Some(ManagedBuffer::from(ERROR_AT_ENCODING)),
            );
            return;
        };
        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &config_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }
        if let Some(lock_operation_error) =
            self.lock_operation_hash_wrapper(&config_hash, &hash_of_hashes)
        {
            self.complete_operation(&hash_of_hashes, &config_hash, Some(lock_operation_error));
            return;
        }

        if let Some(error_message) = self.is_new_config_valid(&new_config) {
            self.complete_operation(&hash_of_hashes, &config_hash, Some(error_message.into()));
            return;
        } else {
            self.sovereign_config().set(new_config);
            self.complete_operation(&hash_of_hashes, &config_hash, None);
        }
    }
}
