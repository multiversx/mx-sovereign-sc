multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FactoryModule {
    // TODO: Typed args
    #[payable("EGLD")]
    #[endpoint(deploySovereignChainConfigContract)]
    fn deploy_sovereign_chain_config_contract(&self, args: MultiValueEncoded<ManagedBuffer>) {
        let payment_amount = self.call_value().egld_value().clone_value();
        let deploy_cost = self.deploy_cost().get();
        require!(payment_amount == deploy_cost, "Invalid payment amount");

        let mut serialized_args = ManagedArgBuffer::new();
        for arg in args {
            serialized_args.push_arg(arg);
        }

        // TODO: add caller as admin
        // let caller = self.blockchain().get_caller();

        // TODO: Typed call based on proxy
        let source_address = self.chain_config_template().get();
        let gas_left = self.blockchain().get_gas_left();
        let metadata =
            CodeMetadata::PAYABLE_BY_SC | CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE;
        let (sc_address, _) = self.send_raw().deploy_from_source_contract(
            gas_left,
            &BigUint::zero(),
            &source_address,
            metadata,
            &serialized_args,
        );
        let _ = self.all_deployed_contracts().insert(sc_address);
    }

    #[only_owner]
    #[endpoint(blacklistSovereignChainSc)]
    fn blacklist_sovereign_chain_sc(&self, sc_address: ManagedAddress) {
        let _ = self.all_deployed_contracts().swap_remove(&sc_address);
    }

    #[view(getDeployCost)]
    #[storage_mapper("deployCost")]
    fn deploy_cost(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("chainConfigTemplate")]
    fn chain_config_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("allDeployedContracts")]
    fn all_deployed_contracts(&self) -> UnorderedSetMapper<ManagedAddress>;
}
