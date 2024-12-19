use transaction::StakeArgs;

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
            !self.min_validators().is_empty(),
            "The minimum number of validators is not set"
        );
        require!(
            !self.max_validators().is_empty(),
            "The maximum number of validators is not set"
        );
        require!(
            !self.min_stake().is_empty(),
            "The mininum number of stake is not set"
        );
    }

    #[inline]
    fn is_new_min_validators_value(&self, new_min_validatrs: u64) -> bool {
        self.min_validators().get() == new_min_validatrs
    }

    #[inline]
    fn is_new_max_validators_value(&self, new_max_validatrs: u64) -> bool {
        self.max_validators().get() == new_max_validatrs
    }

    #[inline]
    fn is_new_min_stake_value(&self, min_stake: &BigUint) -> bool {
        self.min_stake().get() == *min_stake
    }

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
    fn additional_stake_required(&self) -> UnorderedSetMapper<StakeArgs<Self::Api>>;

    #[view(wasPreviouslySlashed)]
    #[storage_mapper("wasPreviouslySlashed")]
    fn was_previously_slashed(&self, validator: &ManagedAddress) -> SingleValueMapper<bool>;
}
