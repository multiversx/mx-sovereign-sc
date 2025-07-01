use error_messages::VALIDATOR_RANGE_EXCEEDED;
use structs::ValidatorInfo;

use crate::{config_utils, storage};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ValidatorModule:
    setup_phase::SetupPhaseModule
    + events::EventsModule
    + storage::ChainConfigStorageModule
    + config_utils::ChainConfigUtilsModule
{
    #[payable]
    #[endpoint(register)]
    fn register(&self, new_bls_key: ManagedBuffer<Self::Api>) {
        self.require_registration_enabled();

        let (egld_stake, additional_stake) = self.validate_stake();

        self.require_validator_not_registered(&new_bls_key);

        let max_number_of_validators = self.sovereign_config().get().max_validators;
        let last_bls_key_id_mapper = self.last_bls_key_id();
        let current_bls_key_id = &last_bls_key_id_mapper.get() + 1u32;

        require!(
            current_bls_key_id <= max_number_of_validators,
            VALIDATOR_RANGE_EXCEEDED
        );

        self.last_bls_key_id().set(current_bls_key_id.clone());
        self.bls_keys_map()
            .insert(current_bls_key_id.clone(), new_bls_key.clone());
        self.bls_key_to_id_mapper(&new_bls_key)
            .set(current_bls_key_id.clone());

        let caller = self.blockchain().get_caller();

        self.register_event(
            &current_bls_key_id,
            &self.blockchain().get_caller(),
            &new_bls_key,
            &egld_stake,
            &additional_stake,
        );

        self.validator_info(&current_bls_key_id).set(ValidatorInfo {
            address: caller,
            bls_key: new_bls_key,
            egld_stake,
            token_stake: additional_stake,
        });
    }

    #[endpoint(unregister)]
    fn unregister(&self, bls_key: ManagedBuffer<Self::Api>) {
        self.require_validator_registered(&bls_key);

        let validator_id = self.bls_key_to_id_mapper(&bls_key).get();
        let validator_info = self.validator_info(&validator_id).get();

        self.bls_keys_map().remove(&validator_id);
        self.bls_key_to_id_mapper(&validator_info.bls_key).clear();
        self.refund_stake(&validator_info);
        self.validator_info(&validator_id).clear();

        self.unregister_event(
            &validator_id,
            &validator_info.address,
            &validator_info.bls_key,
            &validator_info.egld_stake,
            &validator_info.token_stake,
        );
    }
}
