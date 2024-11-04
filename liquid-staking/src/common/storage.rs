use multiversx_sc::imports::*;
pub type Epoch = u64;
pub type BlsKey<M> = ManagedBuffer<M>;
pub type ChainId<M> = ManagedBuffer<M>;

#[multiversx_sc::module]
pub trait CommonStorageModule {
    #[view(getDelegationAddress)]
    #[storage_mapper("delegationAddress")]
    fn delegation_addresses(
        &self,
        contract_name: &ManagedBuffer,
    ) -> SingleValueMapper<ManagedAddress>;

    // TODO: use AddressToIdMapper for lower gas usage
    #[storage_mapper("userIds")]
    fn validator_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[view(getDelegatedValue)]
    #[storage_mapper("delegatedValue")]
    fn delegated_value(&self, validator: &AddressId) -> SingleValueMapper<BigUint>;

    #[view(unDelegateEpoch)]
    #[storage_mapper("unDelegateEpoch")]
    fn undelegate_epoch(&self, address: &ManagedAddress) -> SingleValueMapper<Epoch>;

    #[view(getTotalEgldSupply)]
    #[storage_mapper("totalEgldSupply")]
    fn egld_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[view(getHeaderVerifierAddress)]
    #[storage_mapper("headerVerifierAddress")]
    fn header_verifier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getRegisteredBlsKeys)]
    #[storage_mapper("registeredBlsKeys")]
    fn registered_bls_keys(&self) -> UnorderedSetMapper<BlsKey<Self::Api>>;

    #[view(validatorBlsKeyMap)]
    #[storage_mapper("validatorBlsKeyMap")]
    fn validator_bls_key_address_map(
        &self,
        bls_key: &BlsKey<Self::Api>,
    ) -> SingleValueMapper<ManagedAddress>;

    #[view(getValidatorId)]
    fn get_validator_id(&self, bls_key: &BlsKey<Self::Api>) -> u64 {
        let validator_address = self.validator_bls_key_address_map(bls_key).get();

        self.validator_ids().get_id(&validator_address)
    }

    // NOTE: Number of nodes where ?
    #[view(lockedSupply)]
    #[storage_mapper("lockerSupply")]
    fn locked_supply(&self, chain_id: ChainId<Self::Api>) -> SingleValueMapper<BigUint>;

    fn require_bls_key_registered(&self, bls_key: &BlsKey<Self::Api>) {
        require!(
            self.registered_bls_keys().contains(bls_key),
            "The given bls key is not registered"
        );
    }

    fn require_caller_header_verifier(&self, address: &ManagedAddress) {
        require!(
            !self.header_verifier_address().is_empty(),
            "There is no address registered as the Header Verifier"
        );

        let header_verifier_address = self.header_verifier_address().get();

        require!(
            address == &header_verifier_address,
            "Caller is not Header Verifier contract"
        );
    }

    fn require_address_has_stake(&self, validator_address: &ManagedAddress) {
        let validator_id = self.validator_ids().get_id_or_insert(validator_address);
        let total_egld_deposit = self.delegated_value(&validator_id).get();

        require!(total_egld_deposit > 0, "Caller has 0 delegated value");
    }
}
