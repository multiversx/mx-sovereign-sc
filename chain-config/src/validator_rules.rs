use transaction::SovereignConfig;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

// TODO: What to fill here?
pub enum SlashableOffenses {}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct TokenIdAmountPair<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub amount: BigUint<M>,
}

#[multiversx_sc::module]
pub trait ValidatorRulesModule {
    fn require_config_set(&self) {
        require!(
            !self.sovereign_config().is_empty(),
            "The Sovereign Config is not set"
        );
    }

    fn require_validator_range(&self, min_validators: u64, max_validators: u64) {
        require!(
            min_validators <= max_validators,
            "Invalid min/max validator numbers"
        );
    }

    #[view(sovereignConfig)]
    #[storage_mapper("sovereignConfig")]
    fn sovereign_config(&self) -> SingleValueMapper<SovereignConfig<Self::Api>>;

    #[view(wasPreviouslySlashed)]
    #[storage_mapper("wasPreviouslySlashed")]
    fn was_previously_slashed(&self, validator: &ManagedAddress) -> SingleValueMapper<bool>;
}
