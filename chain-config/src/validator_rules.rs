use error_messages::{
    INVALID_ADDITIONAL_STAKE, INVALID_MIN_MAX_VALIDATOR_NUMBERS, INVALID_TOKEN_ID,
    NOT_ENOUGH_VALIDATORS, VALIDATOR_ALREADY_REGISTERED, VALIDATOR_NOT_REGISTERED,
    VALIDATOR_RANGE_EXCEEDED,
};
use structs::{configs::SovereignConfig, ValidatorInfo};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ValidatorRulesModule: setup_phase::SetupPhaseModule + events::EventsModule {
    fn is_new_config_valid(&self, config: &SovereignConfig<Self::Api>) -> Option<&str> {
        if let Some(additional_stake) = config.opt_additional_stake_required.clone() {
            for stake in additional_stake {
                require!(stake.token_id.is_valid_esdt_identifier(), INVALID_TOKEN_ID);
            }
        }

        if config.min_validators <= config.max_validators {
            None
        } else {
            Some(INVALID_MIN_MAX_VALIDATOR_NUMBERS)
        }
    }

    #[payable]
    #[endpoint(register)]
    fn register(&self, new_validator: ValidatorInfo<Self::Api>) {
        if !self.is_genesis_phase_complete() {
            self.validate_additional_stake();
        }

        self.require_validator_not_registered(&new_validator.bls_key);

        let max_number_of_validators = self.sovereign_config().get().max_validators;
        let last_bls_key_id_mapper = self.last_bls_key_id();
        let current_bls_key_id = &last_bls_key_id_mapper.get() + 1u32;

        require!(
            current_bls_key_id <= max_number_of_validators,
            VALIDATOR_RANGE_EXCEEDED
        );

        self.last_bls_key_id().set(current_bls_key_id.clone());
        self.bls_keys_map()
            .insert(current_bls_key_id.clone(), new_validator.bls_key.clone());
        self.bls_key_to_id_mapper(&new_validator.bls_key)
            .set(current_bls_key_id.clone());

        self.register_event(
            &current_bls_key_id,
            &new_validator.address,
            &new_validator.bls_key,
            &new_validator.egld_stake,
            &new_validator.token_stake,
        );
    }

    #[endpoint(unregister)]
    fn unregister(&self, validator_info: ValidatorInfo<Self::Api>) {
        self.require_setup_complete();
        self.require_validator_registered(&validator_info.bls_key);

        let validator_id = self.bls_key_to_id_mapper(&validator_info.bls_key).get();

        self.bls_keys_map().remove(&validator_id);
        self.bls_key_to_id_mapper(&validator_info.bls_key).clear();

        self.unregister_event(
            &validator_id,
            &validator_info.address,
            &validator_info.bls_key,
            &validator_info.egld_stake,
            &validator_info.token_stake,
        );
    }

    fn require_validator_not_registered(&self, bls_key: &ManagedBuffer) {
        require!(
            self.bls_key_to_id_mapper(bls_key).is_empty(),
            VALIDATOR_ALREADY_REGISTERED
        );
    }

    fn require_validator_registered(&self, bls_key: &ManagedBuffer) {
        require!(
            !self.bls_key_to_id_mapper(bls_key).is_empty(),
            VALIDATOR_NOT_REGISTERED
        );
    }

    fn require_validator_set_valid(&self, validator_len: usize) {
        let config = self.sovereign_config().get();

        require!(
            validator_len as u64 >= config.min_validators,
            NOT_ENOUGH_VALIDATORS
        );
    }

    fn is_genesis_phase_complete(&self) -> bool {
        self.genesis_phase_status().get()
    }

    // TODO: send back tokens if additional stake is not enough
    fn validate_additional_stake(&self) {
        if let Some(additional_stake) = &self.sovereign_config().get().opt_additional_stake_required
        {
            let call_value = self.call_value().all_esdt_transfers();

            for stake in additional_stake {
                let matched = call_value.iter().any(|paid| {
                    paid.token_identifier == stake.token_id && paid.amount >= stake.amount
                });

                require!(matched, INVALID_ADDITIONAL_STAKE);
            }
        }
    }

    #[view(sovereignConfig)]
    #[storage_mapper("sovereignConfig")]
    fn sovereign_config(&self) -> SingleValueMapper<SovereignConfig<Self::Api>>;

    #[view(blsKeyToId)]
    #[storage_mapper("blsKeyToId")]
    fn bls_key_to_id_mapper(
        &self,
        bls_key: &ManagedBuffer,
    ) -> SingleValueMapper<BigUint<Self::Api>>;

    #[view(blsKeysMap)]
    #[storage_mapper("blsKeysMap")]
    fn bls_keys_map(&self) -> MapMapper<BigUint<Self::Api>, ManagedBuffer>;

    #[storage_mapper("lastBlsKeyId")]
    fn last_bls_key_id(&self) -> SingleValueMapper<BigUint<Self::Api>>;

    #[storage_mapper("genesisPhase")]
    fn genesis_phase_status(&self) -> SingleValueMapper<bool>;

    #[view(wasPreviouslySlashed)]
    #[storage_mapper("wasPreviouslySlashed")]
    fn was_previously_slashed(&self, validator: &ManagedAddress) -> SingleValueMapper<bool>;
}
