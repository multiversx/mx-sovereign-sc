use chain_config::StakeMultiArg;

multiversx_sc::imports!();

pub enum ContractName<M: ManagedTypeApi> {
    ChainFactory(ManagedBuffer<M>),
    Controller,
    SovereignHeaderVerifier,
    SovereignCrossChainOperation,
    ChainConfig,
    Slashing,
}

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
                caller.clone(),
                additional_stake_required,
            )
            .deploy_from_source::<IgnoreValue>(&source_address, metadata);

        let _ = self.all_deployed_contracts().insert(sc_address);
        self.config_admin().set(caller);
    }

    #[only_owner]
    #[endpoint(blacklistSovereignChainSc)]
    fn blacklist_sovereign_chain_sc(&self, sc_address: ManagedAddress) {
        let _ = self.all_deployed_contracts().swap_remove(&sc_address);
    }

    #[endpoint(deploySovereignCrossChainOperation)]
    fn deploy_sovereign_cross_chain_operation(&self, _chain_id: u32) {
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.config_admin().get(),
            "This endpoint can only be called by the chain config admin"
        );
        
        // check if contract deployed
    }

    #[endpoint(deploySovereignHeaderVerifier)]
    fn deploy_sovereign_header_verifier(&self, _chain_id: u32) {
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.config_admin().get(),
            "This endpoint can only be called by the chain config admin"
        );

        // check if contract deployed
    }

    // upgrade contract endpoint

    #[endpoint(addContractsToMap)]
    fn add_contracts_to_map(
        &self,
        contracts_map: MultiValueEncoded<MultiValue2<ManagedBuffer, ManagedAddress>>,
        // maybe use enum for contract name?
    ) {
        for contract_map in contracts_map {
            let (contract_name, contract_address) = contract_map.into_tuple();

            self.contracts_map(contract_name).set(contract_address);
        }
    }

    #[proxy]
    fn chain_config_proxy(&self) -> chain_config::Proxy<Self::Api>;

    #[view(getDeployCost)]
    #[storage_mapper("deployCost")]
    fn deploy_cost(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("chainConfigTemplate")]
    fn chain_config_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("allDeployedContracts")]
    fn all_deployed_contracts(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("contractsMap")]
    fn contracts_map(&self, contract_name: ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("configAdmin")]
    fn config_admin(&self) -> SingleValueMapper<ManagedAddress>;
}
