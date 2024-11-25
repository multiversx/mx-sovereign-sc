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
    #[view(getMinValidators)]
    #[storage_mapper("minValidators")]
    fn min_validators(&self) -> SingleValueMapper<u64>;

    #[view(getMaxValidators)]
    #[storage_mapper("maxValidators")]
    fn max_validators(&self) -> SingleValueMapper<u64>;

    // TODO: Read user stake and verify
    #[view(getMinStake)]
    #[storage_mapper("minStake")]
    fn min_stake(&self) -> SingleValueMapper<BigUint>;

    // NOTE: ManagedVec or MultiValueEncoded ?
    // TODO: Read user stake and verify
    #[view(getAdditionalStakeRequired)]
    #[storage_mapper("additionalStakeRequired")]
    fn additional_stake_required(
        &self,
    ) -> SingleValueMapper<ManagedVec<TokenIdAmountPair<Self::Api>>>;

    #[view(wasPreviouslySlashed)]
    #[storage_mapper("wasPreviouslySlashed")]
    fn was_previously_slashed(&self, validator: &ManagedAddress) -> SingleValueMapper<bool>;
}
