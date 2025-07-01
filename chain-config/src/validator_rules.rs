use error_messages::{
    ADDITIONAL_STAKE_NOT_REQUIRED, ADDITIONAL_STAKE_ZERO_VALUE, CALLER_HAS_INCORRECT_BLS_KEY,
    EMPTY_ADDITIONAL_STAKE, INVALID_ADDITIONAL_STAKE, INVALID_EGLD_STAKE,
    INVALID_MIN_MAX_VALIDATOR_NUMBERS, INVALID_TOKEN_ID, NOT_ENOUGH_VALIDATORS,
    REGISTRATION_DISABLED, VALIDATOR_ALREADY_REGISTERED, VALIDATOR_NOT_REGISTERED,
    VALIDATOR_RANGE_EXCEEDED,
};
use multiversx_sc::chain_core::EGLD_000000_TOKEN_IDENTIFIER;
use structs::{configs::SovereignConfig, ValidatorInfo};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ValidatorRulesModule: setup_phase::SetupPhaseModule + events::EventsModule {
    fn is_new_config_valid(&self, config: &SovereignConfig<Self::Api>) -> Option<&str> {
        if let Some(additional_stake) = config.opt_additional_stake_required.clone() {
            require!(!additional_stake.is_empty(), EMPTY_ADDITIONAL_STAKE);
            for stake in additional_stake {
                require!(
                    stake.token_identifier.is_valid_esdt_identifier(),
                    INVALID_TOKEN_ID
                );
                require!(stake.amount > 0, ADDITIONAL_STAKE_ZERO_VALUE);
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

        self.require_caller_has_bls_key(self.blockchain().get_caller(), &validator_id);

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

    fn refund_stake(&self, validator_info: &ValidatorInfo<Self::Api>) {
        self.tx()
            .to(&self.blockchain().get_caller())
            .payment(self.get_total_stake(validator_info))
            .transfer_execute();
    }

    fn get_total_stake(
        &self,
        validator_info: &ValidatorInfo<Self::Api>,
    ) -> MultiEgldOrEsdtPayment<Self::Api> {
        let mut total_stake = MultiEgldOrEsdtPayment::new();
        total_stake.push(EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::from(ManagedBuffer::from(EGLD_000000_TOKEN_IDENTIFIER)),
            0,
            validator_info.egld_stake.clone(),
        ));

        if let Some(additional_stake) = &validator_info.token_stake {
            for stake in additional_stake {
                total_stake.push(stake.clone().into());
            }
        }

        total_stake
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

    fn require_caller_has_bls_key(
        &self,
        caller: ManagedAddress<Self::Api>,
        id: &BigUint<Self::Api>,
    ) {
        require!(
            self.validator_info(id).get().address == caller,
            CALLER_HAS_INCORRECT_BLS_KEY
        );
    }

    fn require_validator_set_valid(&self, validator_len: usize) {
        let config = self.sovereign_config().get();

        require!(
            validator_len as u64 >= config.min_validators,
            NOT_ENOUGH_VALIDATORS
        );
    }

    fn validate_stake(
        &self,
    ) -> (
        BigUint<Self::Api>,
        Option<ManagedVec<EsdtTokenPayment<Self::Api>>>,
    ) {
        let sovereign_config = self.sovereign_config().get();

        let (egld_amount, esdt_payments) = self.split_payments();

        require!(
            egld_amount == sovereign_config.min_stake,
            INVALID_EGLD_STAKE
        );

        if let Some(additional) = &sovereign_config.opt_additional_stake_required {
            let valid = additional.iter().all(|s| {
                esdt_payments
                    .iter()
                    .any(|p| p.token_identifier == s.token_identifier && p.amount == s.amount)
            });
            require!(valid, INVALID_ADDITIONAL_STAKE);
        } else {
            require!(esdt_payments.is_empty(), ADDITIONAL_STAKE_NOT_REQUIRED);
        }

        (egld_amount, Some(esdt_payments))
    }

    fn split_payments(&self) -> (BigUint, ManagedVec<EsdtTokenPayment<Self::Api>>) {
        let mut egld_amount = BigUint::zero();
        let mut esdt_payments = ManagedVec::new();

        for payment in self.call_value().all_transfers().clone_value().into_iter() {
            if payment.token_identifier.is_egld() {
                egld_amount = payment.amount.clone();
            } else {
                esdt_payments.push(payment.unwrap_esdt());
            }
        }

        (egld_amount, esdt_payments)
    }

    fn require_registration_enabled(&self) {
        require!(self.registration_status().get() == 1, REGISTRATION_DISABLED);
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

    #[view(validatorInfo)]
    #[storage_mapper("validatorInfo")]
    fn validator_info(
        &self,
        id: &BigUint<Self::Api>,
    ) -> SingleValueMapper<ValidatorInfo<Self::Api>>;

    #[view(blsKeysMap)]
    #[storage_mapper("blsKeysMap")]
    fn bls_keys_map(&self) -> MapMapper<BigUint<Self::Api>, ManagedBuffer>;

    #[storage_mapper("lastBlsKeyId")]
    fn last_bls_key_id(&self) -> SingleValueMapper<BigUint<Self::Api>>;

    #[storage_mapper("registration_status")]
    fn registration_status(&self) -> SingleValueMapper<u8>;

    #[view(wasPreviouslySlashed)]
    #[storage_mapper("wasPreviouslySlashed")]
    fn was_previously_slashed(&self, validator: &ManagedAddress) -> SingleValueMapper<bool>;
}
