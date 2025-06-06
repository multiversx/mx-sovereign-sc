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
    fn is_new_config_valid(&self, config: &SovereignConfig<Self::Api>) -> Option<&str> {
        if config.min_validators <= config.max_validators {
            None
        } else {
            Some(INVALID_MIN_MAX_VALIDATOR_NUMBERS)
        }
    }

    #[view(sovereignConfig)]
    #[storage_mapper("sovereignConfig")]
    fn sovereign_config(&self) -> SingleValueMapper<SovereignConfig<Self::Api>>;

    #[view(wasPreviouslySlashed)]
    #[storage_mapper("wasPreviouslySlashed")]
    fn was_previously_slashed(&self, validator: &ManagedAddress) -> SingleValueMapper<bool>;
}
