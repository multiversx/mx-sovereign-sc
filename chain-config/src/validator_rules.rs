use crate::liquid_staking_proxy;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

// TODO: What to fill here?
pub enum SlashableOffenses {}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct TokenIdAmountPair<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub amount: BigUint<M>,
}

#[multiversx_sc::module]
pub trait ValidatorRulesModule {
    #[view(getMinBlsKeys)]
    #[storage_mapper("minBlsKeys")]
    fn min_bls_keys(&self) -> SingleValueMapper<usize>;

    #[view(getMaxBlsKeys)]
    #[storage_mapper("maxBlsKeys")]
    fn max_bls_keys(&self) -> SingleValueMapper<usize>;

    #[view(getBlsWhitelist)]
    #[storage_mapper("blsWhitelist")]
    fn bls_whitelist(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[view(getBlsBlacklist)]
    #[storage_mapper("blsBlacklist")]
    fn bls_blacklist(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[view(getBlsKeys)]
    #[storage_mapper("blsKeys")]
    fn bls_keys(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[view(getMinValidators)]
    #[storage_mapper("minValidators")]
    fn min_validators(&self) -> SingleValueMapper<usize>;

    #[view(getMaxValidators)]
    #[storage_mapper("maxValidators")]
    fn max_validators(&self) -> SingleValueMapper<usize>;

    // TODO: Read user stake and verify
    #[view(getMinStake)]
    #[storage_mapper("minStake")]
    fn min_stake(&self) -> SingleValueMapper<BigUint>;

    // TODO: Read user stake and verify
    #[view(getAdditionalStakeRequired)]
    #[storage_mapper("additionalStakeRequired")]
    fn additional_stake_required(
        &self,
    ) -> SingleValueMapper<ManagedVec<TokenIdAmountPair<Self::Api>>>;

    #[view(wasPreviouslySlashed)]
    #[storage_mapper("wasPreviouslySlashed")]
    fn was_previously_slashed(&self, validator: &ManagedAddress) -> SingleValueMapper<bool>;

    #[view(getAddressWhitelist)]
    #[storage_mapper("addressWhitelist")]
    fn address_whitelist(&self) -> UnorderedSetMapper<ManagedAddress<Self::Api>>;

    #[view(getAddressBlacklist)]
    #[storage_mapper("addressBlacklist")]
    fn address_blacklist(&self) -> UnorderedSetMapper<ManagedAddress<Self::Api>>;

    #[view(getNativeTokenId)]
    #[storage_mapper("nativeTokenId")]
    fn native_token_id(&self) -> SingleValueMapper<TokenIdentifier<Self::Api>>;

    #[view(getHeaderVerifierAddress)]
    #[storage_mapper("headerVerifierAddress")]
    fn header_verifier_address(&self) -> SingleValueMapper<ManagedAddress<Self::Api>>;

    #[view(getLiquidStakingAddress)]
    #[storage_mapper("liquidStakingAddress")]
    fn liquid_staking_address(&self) -> SingleValueMapper<ManagedAddress<Self::Api>>;

    // TODO: use AddressToId Mapper when fix is here
    // #[storage_mapper_from_address("userIds")]
    // fn external_validator_ids(
    //     &self,
    //     sc_address: ManagedAddress,
    // ) -> AddressToIdMapper<Self::Api, ManagedAddress>;

    #[inline]
    fn require_bls_key_whitelist(&self, bls_key: &ManagedBuffer) {
        require!(
            self.bls_whitelist().contains(bls_key),
            "BLS key is not whitelisted"
        )
    }

    fn get_delegated_value(&self, validator_id: u64) -> BigUint<Self::Api> {
        let liquid_staking_address = self.liquid_staking_address().get();

        self.tx()
            .to(liquid_staking_address)
            .typed(liquid_staking_proxy::LiquidStakingProxy)
            .delegated_value(validator_id)
            .returns(ReturnsResultUnmanaged)
            .sync_call()
            .into()
    }

    fn has_stake_in_validator_sc(&self, bls_key: &ManagedBuffer) -> bool {
        let liquid_staking_address = self.liquid_staking_address().get();
        let validator_id = self
            .tx()
            .to(liquid_staking_address)
            .typed(liquid_staking_proxy::LiquidStakingProxy)
            .get_validator_id(bls_key)
            .returns(ReturnsResultUnmanaged)
            .sync_call();

        if validator_id != 0 {
            return true;
        }

        false
    }

    fn require_bls_keys_length_limits(&self, length: usize) {
        let min_bls_keys = self.min_bls_keys().get();
        let max_bls_keys = self.max_bls_keys().get();

        require!(
            length > min_bls_keys,
            "There are fewer BLS public keys than expected"
        );
        require!(
            length < max_bls_keys,
            "There are fewer BLS public keys than expected"
        );
    }

    #[inline]
    fn require_min_stake(&self, amount: BigUint) {
        let min_amount = self.min_stake().get();
        require!(amount > min_amount, "Minimum stake amount not met");
    }
}
