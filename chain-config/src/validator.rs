use crate::{config_utils, storage};
use error_messages::{
    INVALID_VALIDATOR_DATA, REGISTRATIONS_DISABLED_GENESIS_PHASE, VALIDATOR_ID_NOT_REGISTERED,
};
use structs::generate_hash::GenerateHash;
use structs::{ValidatorInfo, ValidatorOperation};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ValidatorModule:
    setup_phase::SetupPhaseModule
    + common_utils::CommonUtilsModule
    + custom_events::CustomEventsModule
    + storage::ChainConfigStorageModule
    + config_utils::ChainConfigUtilsModule
{
    #[payable]
    #[endpoint(register)]
    fn register(&self, bls_key: ManagedBuffer) {
        if self.is_setup_phase_complete() {
            sc_panic!(REGISTRATIONS_DISABLED_GENESIS_PHASE);
        }

        self.require_valid_bls_key(&bls_key);
        self.require_validator_not_registered(&bls_key);

        let current_bls_key_id = self.get_bls_key_id();

        let (egld_stake, additional_stake) = self.validate_stake();

        self.insert_validator(
            current_bls_key_id.clone(),
            &ValidatorInfo {
                address: self.blockchain().get_caller(),
                bls_key,
                egld_stake,
                token_stake: additional_stake,
            },
        );
    }

    #[inline]
    fn get_bls_key_id(&self) -> BigUint {
        let max_number_of_validators = self.sovereign_config().get().max_validators;
        let last_bls_key_id_mapper = self.last_bls_key_id();
        let current_bls_key_id = &last_bls_key_id_mapper.get() + 1u32;
        self.last_bls_key_id().set(current_bls_key_id.clone());

        self.require_valid_validator_range(&current_bls_key_id, max_number_of_validators);

        current_bls_key_id
    }

    #[endpoint(registerBlsKey)]
    fn register_bls_key(
        &self,
        hash_of_hashes: ManagedBuffer,
        validator_operation: ValidatorOperation<Self::Api>,
    ) {
        let config_hash = validator_operation.generate_hash();
        if let Some(lock_operation_error) = self.lock_operation_hash_wrapper(
            &hash_of_hashes,
            &config_hash,
            validator_operation.nonce,
        ) {
            self.complete_operation(&hash_of_hashes, &config_hash, Some(lock_operation_error));
            return;
        }

        self.insert_validator(
            validator_operation.validator_data.id,
            &ValidatorInfo {
                address: validator_operation.validator_data.address,
                bls_key: validator_operation.validator_data.bls_key,
                egld_stake: BigUint::zero(),
                token_stake: None,
            },
        );

        self.complete_operation(&hash_of_hashes, &config_hash, None);
    }

    #[inline]
    fn insert_validator(&self, id: BigUint, validator_info: &ValidatorInfo<Self::Api>) {
        self.bls_keys_map()
            .insert(id.clone(), validator_info.bls_key.clone());
        self.bls_key_to_id_mapper(&validator_info.bls_key)
            .set(id.clone());
        self.validator_info(&id).set(validator_info);
    }

    #[endpoint(unregister)]
    fn unregister(&self, bls_key: ManagedBuffer) {
        if self.is_setup_phase_complete() {
            sc_panic!(REGISTRATIONS_DISABLED_GENESIS_PHASE);
        }

        self.require_valid_bls_key(&bls_key);
        self.require_validator_registered(&bls_key);

        let caller = self.blockchain().get_caller();
        let validator_id = self.bls_key_to_id_mapper(&bls_key).get();
        let validator_info = self.validator_info(&validator_id).get();

        self.require_caller_has_bls_key(&caller, &validator_info);

        self.remove_validator(validator_id, &validator_info);

        self.refund_stake(&caller, &validator_info);
    }

    #[endpoint(unregisterBlsKey)]
    fn unregister_bls_key(
        &self,
        hash_of_hashes: ManagedBuffer,
        validator_operation: ValidatorOperation<Self::Api>,
    ) {
        let config_hash = validator_operation.generate_hash();
        if let Some(lock_operation_error) = self.lock_operation_hash_wrapper(
            &hash_of_hashes,
            &config_hash,
            validator_operation.nonce,
        ) {
            self.complete_operation(&hash_of_hashes, &config_hash, Some(lock_operation_error));
            return;
        }

        let validator_info_mapper = self.validator_info(&validator_operation.validator_data.id);
        if validator_info_mapper.is_empty() {
            self.complete_operation(
                &hash_of_hashes,
                &config_hash,
                Some(VALIDATOR_ID_NOT_REGISTERED.into()),
            );
            return;
        }
        let validator_info = self
            .validator_info(&validator_operation.validator_data.id)
            .get();

        if validator_operation.validator_data.address != validator_info.address {
            self.complete_operation(
                &hash_of_hashes,
                &config_hash,
                Some(INVALID_VALIDATOR_DATA.into()),
            );
            return;
        }

        self.remove_validator(validator_operation.validator_data.id, &validator_info);

        self.refund_stake(&validator_operation.validator_data.address, &validator_info);

        self.complete_operation(&hash_of_hashes, &config_hash, None);
    }

    #[inline]
    fn remove_validator(&self, id: BigUint, validator_info: &ValidatorInfo<Self::Api>) {
        self.bls_keys_map().remove(&id);
        self.bls_key_to_id_mapper(&validator_info.bls_key).clear();
        self.validator_info(&id).clear();
    }
}
