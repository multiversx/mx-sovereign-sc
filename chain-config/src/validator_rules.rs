use error_messages::INVALID_MIN_MAX_VALIDATOR_NUMBERS;
use structs::configs::SovereignConfig;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct TokenIdAmountPair<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub amount: BigUint<M>,
}

#[multiversx_sc::module]
pub trait ValidatorRulesModule {
    fn require_valid_config(&self, config: &SovereignConfig<Self::Api>) {
        // TODO: determine a range value
        self.require_validator_range(config.min_validators, config.max_validators);
    }

    fn require_validator_range(&self, min_validators: u64, max_validators: u64) {
        require!(
            min_validators <= max_validators,
            INVALID_MIN_MAX_VALIDATOR_NUMBERS
        );
    }

    #[view(sovereignConfig)]
    #[storage_mapper("sovereignConfig")]
    fn sovereign_config(&self) -> SingleValueMapper<SovereignConfig<Self::Api>>;

    #[view(wasPreviouslySlashed)]
    #[storage_mapper("wasPreviouslySlashed")]
    fn was_previously_slashed(&self, validator: &ManagedAddress) -> SingleValueMapper<bool>;
}
