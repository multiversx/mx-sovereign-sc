use multiversx_sc::imports::*;
pub type Epoch = u64;

#[multiversx_sc::module]
pub trait CommonStorageModule {
    #[view(getDelegationAddress)]
    #[storage_mapper("delegationAddress")]
    fn delegation_addresses(
        &self,
        contract_name: &ManagedBuffer,
    ) -> SingleValueMapper<ManagedAddress>;

    #[view(getDelegatedValue)]
    #[storage_mapper("delegatedValue")]
    fn delegated_value(&self, validator: &ManagedAddress) -> SingleValueMapper<BigUint<Self::Api>>;

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
    fn registered_bls_keys(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[view(validatorBlsKeyMap)]
    #[storage_mapper("validatorBlsKeyMap")]
    fn validator_bls_key_address_map(
        &self,
        bls_key: &ManagedBuffer,
    ) -> SingleValueMapper<ManagedAddress>;

    fn require_bls_key_to_be_registered(&self, bls_key: &ManagedBuffer) {
        require!(
            self.registered_bls_keys().contains(bls_key),
            "The given bls key is not registered"
        );
    }

    fn require_caller_to_be_header_verifier(&self, caller: &ManagedAddress) {
        require!(
            !self.header_verifier_address().is_empty(),
            "There is no address registered as the Header Verifier"
        );

        let header_verifier_address = self.header_verifier_address().get();

        require!(
            caller == &header_verifier_address,
            "Caller is not Header Verifier contract"
        );
    }

    fn require_caller_has_stake(&self, caller: &ManagedAddress) {
        let total_egld_deposit = self.delegated_value(caller).get();

        require!(total_egld_deposit > 0, "Caller has 0 delegated value");
    }
}
