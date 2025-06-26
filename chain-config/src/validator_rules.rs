use error_messages::{
    EMPTY_ADDITIONAL_STAKE, INVALID_ADDITIONAL_STAKE, INVALID_EGLD_STAKE,
    INVALID_MIN_MAX_VALIDATOR_NUMBERS, INVALID_TOKEN_ID, NOT_ENOUGH_VALIDATORS,
    REGISTRATION_PAUSED, VALIDATOR_ALREADY_REGISTERED, VALIDATOR_NOT_REGISTERED,
    VALIDATOR_RANGE_EXCEEDED,
};
use structs::{
    configs::{SovereignConfig, StakeArgs},
    ValidatorInfo,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ValidatorRulesModule: setup_phase::SetupPhaseModule + events::EventsModule {
    fn is_new_config_valid(&self, config: &SovereignConfig<Self::Api>) -> Option<&str> {
        if let Some(additional_stake) = config.opt_additional_stake_required.clone() {
            require!(!additional_stake.is_empty(), EMPTY_ADDITIONAL_STAKE);
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
    fn register(&self, new_bls_key: ManagedBuffer<Self::Api>) {
        if !self.is_genesis_phase_active() {
            self.require_registration_not_frozen();
        }

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

        self.register_event(
            &current_bls_key_id,
            &self.blockchain().get_caller(),
            &new_bls_key,
            &egld_stake,
            &additional_stake,
        );
    }

    // TODO: add storage for registered stake in order to return it upon unregistering
    #[endpoint(unregister)]
    fn unregister(&self, validator_info: ValidatorInfo<Self::Api>) {
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

    fn is_genesis_phase_active(&self) -> bool {
        self.genesis_phase_status().get()
    }

    // TODO: send back tokens if additional stake is not enough
    fn validate_stake(&self) -> (BigUint<Self::Api>, Option<ManagedVec<StakeArgs<Self::Api>>>) {
        let sovereign_config = self.sovereign_config().get();

        let call_value = self.call_value().all_transfers().clone_value();

        let egld_payment = call_value
            .iter()
            .find(|p| p.token_identifier.is_egld() && p.amount >= sovereign_config.min_stake);

        require!(egld_payment.is_some(), INVALID_EGLD_STAKE);

        let egld_amount = &egld_payment.unwrap().amount;

        if let Some(additional_stakes) = &sovereign_config.opt_additional_stake_required {
            let matched = additional_stakes.iter().all(|s| {
                call_value
                    .iter()
                    .any(|c| c.token_identifier == s.token_id && c.amount >= s.amount)
            });

            require!(matched, INVALID_ADDITIONAL_STAKE);
        }

        (
            egld_amount.clone(),
            sovereign_config.opt_additional_stake_required,
        )
    }

    fn require_registration_not_frozen(&self) {
        require!(self.registration_status().get() == 1, REGISTRATION_PAUSED);
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

    #[storage_mapper("registration_status")]
    fn registration_status(&self) -> SingleValueMapper<u8>;

    #[view(wasPreviouslySlashed)]
    #[storage_mapper("wasPreviouslySlashed")]
    fn was_previously_slashed(&self, validator: &ManagedAddress) -> SingleValueMapper<bool>;
}
