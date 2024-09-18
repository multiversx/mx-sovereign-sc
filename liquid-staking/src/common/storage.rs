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
    fn delegated_value(&self, validator: ManagedAddress) -> SingleValueMapper<BigUint<Self::Api>>;

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

    fn require_caller_to_be_header_verifier(&self, caller: &ManagedAddress) {
        let header_verifier_address = self.header_verifier_address().get();

        require!(
            caller == &header_verifier_address,
            "Caller is not Header Verifier contract"
        );
    }
}
