use multiversx_sc::imports::*;

use crate::factory::{ContractMapArgs, ScArray};

#[multiversx_sc::module]
pub trait CommonStorage {
    #[view(getDeployCost)]
    #[storage_mapper("deployCost")]
    fn deploy_cost(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("chainConfigTemplate")]
    fn chain_config_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("headerVerifierTemplate")]
    fn header_verifier_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("crossChainOperationsTemplate")]
    fn cross_chain_operations_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("feeMarketTemplate")]
    fn fee_market_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("tokenHandlerTemplate")]
    fn token_handler_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("allDeployedContracts")]
    fn all_deployed_contracts(
        &self,
        caller: &ManagedAddress,
    ) -> SingleValueMapper<ContractMapArgs<Self::Api>>;

    #[storage_mapper_from_address("minValidators")]
    fn external_min_validators(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<BigUint<Self::Api>, ManagedAddress>;

    #[storage_mapper_from_address("maxValidators")]
    fn external_max_validators(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<BigUint<Self::Api>, ManagedAddress>;

    #[view(getAllChainIds)]
    #[storage_mapper("allChainIds")]
    fn chain_ids(&self) -> UnorderedSetMapper<ManagedBuffer>;
}
