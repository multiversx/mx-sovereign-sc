use error_messages::{
    INVALID_MIN_MAX_VALIDATOR_NUMBERS, VALIDATOR_ALREADY_REGISTERED, VALIDATOR_NOT_REGISTERED,
    VALIDATOR_RANGE_EXCEEDED,
};
use structs::{configs::SovereignConfig, ValidatorInfo};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct TokenIdAmountPair<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub amount: BigUint<M>,
}

#[multiversx_sc::module]
pub trait ValidatorRulesModule: setup_phase::SetupPhaseModule + events::EventsModule {
    fn is_new_config_valid(&self, config: &SovereignConfig<Self::Api>) -> Option<&str> {
        if config.min_validators <= config.max_validators {
            None
        } else {
            Some(INVALID_MIN_MAX_VALIDATOR_NUMBERS)
        }
    }

    #[endpoint(register)]
    fn register(&self, new_validator: ValidatorInfo<Self::Api>) {
        self.require_setup_complete();
        self.require_validator_not_registered(&new_validator.bls_key);

        let max_number_of_validators = self.sovereign_config().get().max_validators;
        let last_bls_key_id = self.last_bls_key_id().get();
        let current_bls_key_id = &last_bls_key_id + 1u32;

        require!(
            current_bls_key_id <= max_number_of_validators,
            VALIDATOR_RANGE_EXCEEDED
        );

        self.last_bls_key_id().set(current_bls_key_id.clone());
        self.id_to_bls_key_mapper(&current_bls_key_id)
            .set(new_validator.bls_key.clone());
        self.bls_key_to_id_mapper(&new_validator.bls_key)
            .set(current_bls_key_id);

        self.register_event(
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

        let min_number_of_validators = self.sovereign_config().get().min_validators;
        let last_bls_key_id = self.last_bls_key_id().get();
        let current_bls_key_id = &last_bls_key_id - 1u32;

        require!(
            current_bls_key_id > min_number_of_validators,
            VALIDATOR_RANGE_EXCEEDED
        );

        self.last_bls_key_id().set(current_bls_key_id.clone());
        self.id_to_bls_key_mapper(&current_bls_key_id)
            .set(validator_info.bls_key.clone());
        self.bls_key_to_id_mapper(&validator_info.bls_key)
            .set(current_bls_key_id);

        self.unregister_event(
            &validator_info.address,
            &validator_info.bls_key,
            &validator_info.egld_stake,
            &validator_info.token_stake,
        );
    }

    fn require_validator_not_registered(&self, bls_key: &ManagedBuffer) {
        require!(
            self.bls_key_to_id_mapper(bls_key).is_empty(),
            VALIDATOR_NOT_REGISTERED
        );
    }

    fn require_validator_registered(&self, bls_key: &ManagedBuffer) {
        require!(
            !self.bls_key_to_id_mapper(bls_key).is_empty(),
            VALIDATOR_ALREADY_REGISTERED
        );
    }

    #[view(sovereignConfig)]
    #[storage_mapper("sovereignConfig")]
    fn sovereign_config(&self) -> SingleValueMapper<SovereignConfig<Self::Api>>;

    #[view(idToBlsKey)]
    #[storage_mapper("idToBlsKey")]
    fn id_to_bls_key_mapper(&self, id: &BigUint<Self::Api>) -> SingleValueMapper<ManagedBuffer>;

    #[view(blsKeyToId)]
    #[storage_mapper("blsKeyToId")]
    fn bls_key_to_id_mapper(&self, id: &ManagedBuffer) -> SingleValueMapper<BigUint<Self::Api>>;

    #[storage_mapper("lastBlsKeyId")]
    fn last_bls_key_id(&self) -> SingleValueMapper<BigUint<Self::Api>>;

    #[view(wasPreviouslySlashed)]
    #[storage_mapper("wasPreviouslySlashed")]
    fn was_previously_slashed(&self, validator: &ManagedAddress) -> SingleValueMapper<bool>;
}
