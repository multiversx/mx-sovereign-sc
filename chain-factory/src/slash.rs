multiversx_sc::imports!();

pub type DestAmountPairs<M> = MultiValueEncoded<M, MultiValue2<ManagedAddress<M>, BigUint<M>>>;

mod validators_contract_proxy {
    use super::DestAmountPairs;

    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait ValidatorsContractProxy {
        #[endpoint]
        fn slash(&self, validator_address: ManagedAddress, value: BigUint);

        #[endpoint(distributeSlashed)]
        fn distribute_slashed(&self, dest_amount_pairs: DestAmountPairs<Self::Api>);
    }
}

#[multiversx_sc::module]
pub trait SlashModule: crate::factory::FactoryModule {
    #[endpoint]
    fn slash(&self, chain_id: ManagedBuffer, validator_address: ManagedAddress, value: BigUint) {
        let caller = self.blockchain().get_caller();
        self.require_deployed_sc(chain_id, &caller);

        let validators_contract_address = self.validators_contract_address().get();
        let _: IgnoreValue = self
            .validator_proxy(validators_contract_address)
            .slash(validator_address, value)
            .execute_on_dest_context();
    }

    #[endpoint(distributeSlashed)]
    fn distribute_slashed(
        &self,
        chain_id: ManagedBuffer,
        dest_amount_pairs: DestAmountPairs<Self::Api>,
    ) {
        let caller = self.blockchain().get_caller();
        self.require_deployed_sc(chain_id, &caller);

        let validators_contract_address = self.validators_contract_address().get();
        let _: IgnoreValue = self
            .validator_proxy(validators_contract_address)
            .distribute_slashed(dest_amount_pairs)
            .execute_on_dest_context();
    }

    fn require_deployed_sc(&self, chain_id: ManagedBuffer, address: &ManagedAddress) {
        require!(
            self.all_deployed_contracts(chain_id).contains(address),
            "Only deployed contracts may call this endpoint"
        );
    }

    #[proxy]
    fn validator_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> validators_contract_proxy::Proxy<Self::Api>;

    #[storage_mapper("validatorsContractAddress")]
    fn validators_contract_address(&self) -> SingleValueMapper<ManagedAddress>;
}
