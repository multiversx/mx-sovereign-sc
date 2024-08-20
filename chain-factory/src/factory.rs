use chain_config::StakeMultiArg;

multiversx_sc::derive_imports!();
multiversx_sc::imports!();

#[derive(TopEncode, TopDecode, TypeAbi, Clone, ManagedVecItem)]
struct ContractMapArgs<M: ManagedTypeApi> {
    name: ManagedBuffer<M>,
    address: ManagedAddress<M>,
}

const SC_ARRAY: &[&[u8]] = &[
    b"chainFactory",
    b"controller",
    b"sovereignHeaderVerifier",
    b"sovereignCrossChainOperation",
    b"chainConfig",
    b"slashing",
];

#[multiversx_sc::module]
pub trait FactoryModule {
    #[payable("EGLD")]
    #[endpoint(deploySovereignChainConfigContract)]
    fn deploy_sovereign_chain_config_contract(
        &self,
        min_validators: usize,
        max_validators: usize,
        min_stake: BigUint,
        additional_stake_required: MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) {
        let payment_amount = self.call_value().egld_value().clone_value();
        let deploy_cost = self.deploy_cost().get();
        require!(payment_amount == deploy_cost, "Invalid payment amount");

        let caller = self.blockchain().get_caller();
        let source_address = self.chain_config_template().get();
        let metadata =
            CodeMetadata::PAYABLE_BY_SC | CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE;

        let (sc_address, _) = self
            .chain_config_proxy()
            .init(
                min_validators,
                max_validators,
                min_stake,
                caller,
                additional_stake_required,
            )
            .deploy_from_source::<IgnoreValue>(&source_address, metadata);

        let _ = self.all_deployed_contracts().insert(sc_address);
    }

    #[only_owner]
    #[endpoint(addContractsToMap)]
    fn add_contracts_to_map(
        &self,
        contracts_map: MultiValueEncoded<Self::Api, ContractMapArgs<Self::Api>>,
    ) {
        require!(!contracts_map.is_empty(), "Given contracts map is empty");
        let mapped_array: ManagedVec<ManagedBuffer> = SC_ARRAY
            .into_iter()
            .map(|sc| ManagedBuffer::from(*sc))
            .collect();

        for contract_info in contracts_map {
            if mapped_array.contains(&contract_info.name) {
                self.contracts_map(contract_info.name)
                    .set(contract_info.address);
            }
        }
    }

    #[only_owner]
    #[endpoint(blacklistSovereignChainSc)]
    fn blacklist_sovereign_chain_sc(&self, sc_address: ManagedAddress) {
        let _ = self.all_deployed_contracts().swap_remove(&sc_address);
    }

    #[proxy]
    fn chain_config_proxy(&self) -> chain_config::Proxy<Self::Api>;

    #[view(getContractsMap)]
    #[storage_mapper("contractsMap")]
    fn contracts_map(&self, contract_name: ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[view(getDeployCost)]
    #[storage_mapper("deployCost")]
    fn deploy_cost(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("chainConfigTemplate")]
    fn chain_config_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("allDeployedContracts")]
    fn all_deployed_contracts(&self) -> UnorderedSetMapper<ManagedAddress>;
}
