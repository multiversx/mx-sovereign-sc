use error_messages::{
    INVALID_BLS_KEY_PROVIDED, NOT_ENOUGH_VALIDATORS, VALIDATOR_ALREADY_REGISTERED,
    VALIDATOR_NOT_REGISTERED, VALIDATOR_RANGE_EXCEEDED,
};
use structs::{configs::SovereignConfig, ValidatorInfo, BLS_KEY_BYTE_LENGTH};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ChainConfigStorageModule {
    fn require_valid_bls_key(&self, bls_key: &ManagedBuffer) {
        require!(
            bls_key.len() == BLS_KEY_BYTE_LENGTH,
            INVALID_BLS_KEY_PROVIDED
        );
    }

    fn require_validator_not_registered(&self, bls_key: &ManagedBuffer) {
        require!(
            self.bls_key_to_id_mapper(bls_key).is_empty(),
            VALIDATOR_ALREADY_REGISTERED
        );
    }

    fn require_valid_validator_range(
        &self,
        current_bls_key_id: &BigUint<Self::Api>,
        max_number_of_validators: u64,
    ) {
        require!(
            *current_bls_key_id <= max_number_of_validators,
            VALIDATOR_RANGE_EXCEEDED
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

    #[view(sovereignConfig)]
    #[storage_mapper("sovereignConfig")]
    fn sovereign_config(&self) -> SingleValueMapper<SovereignConfig<Self::Api>>;

    #[view(blsKeyToId)]
    #[storage_mapper("blsKeyToId")]
    fn bls_key_to_id_mapper(
        &self,
        bls_key: &ManagedBuffer,
    ) -> SingleValueMapper<BigUint<Self::Api>>;

    #[view(validator_info)]
    #[storage_mapper("validator_info")]
    fn validator_info(
        &self,
        id: &BigUint<Self::Api>,
    ) -> SingleValueMapper<ValidatorInfo<Self::Api>>;

    #[view(blsKeysMap)]
    #[storage_mapper("blsKeysMap")]
    fn bls_keys_map(&self) -> MapMapper<BigUint<Self::Api>, ManagedBuffer>;

    #[storage_mapper("lastBlsKeyId")]
    fn last_bls_key_id(&self) -> SingleValueMapper<BigUint<Self::Api>>;

    #[view(wasPreviouslySlashed)]
    #[storage_mapper("wasPreviouslySlashed")]
    fn was_previously_slashed(&self, validator: &ManagedAddress) -> SingleValueMapper<bool>;
}
