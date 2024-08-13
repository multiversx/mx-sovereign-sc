use chain_config::StakeMultiArg;

multiversx_sc::imports!();

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

        let source_address = self.chain_config_template().get();
        let metadata =
            CodeMetadata::PAYABLE_BY_SC | CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE;

        let args = self.format_args_for_deploy(
            min_validators,
            max_validators,
            min_stake,
            additional_stake_required,
        );

        let chain_config_address = self
            .tx()
            .raw_deploy()
            .gas(self.blockchain().get_gas_left())
            .from_source(source_address)
            .code_metadata(metadata)
            .arguments_raw(args)
            .returns(ReturnsNewManagedAddress)
            .sync_call();

        let _ = self.all_deployed_contracts().insert(chain_config_address);
    }

    fn format_args_for_deploy(
        &self,
        min_validators: usize,
        max_validators: usize,
        min_stake: BigUint,
        additional_stake_required: MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) -> ManagedArgBuffer<Self::Api> {
        let mut args = ManagedArgBuffer::new();
        args.push_arg(min_validators);
        args.push_arg(max_validators);
        args.push_arg(min_stake);
        for stake_required in additional_stake_required {
            let (token_identifier, amount) = stake_required.into_tuple();

            args.push_arg(token_identifier);
            args.push_arg(amount);
        }

        args
    }

    #[only_owner]
    #[endpoint(blacklistSovereignChainSc)]
    fn blacklist_sovereign_chain_sc(&self, sc_address: ManagedAddress) {
        let _ = self.all_deployed_contracts().swap_remove(&sc_address);
    }

    #[storage_mapper("chainConfigAddress")]
    fn chain_config_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn chain_config_proxy(&self) -> chain_config::Proxy<Self::Api>;

    #[view(getDeployCost)]
    #[storage_mapper("deployCost")]
    fn deploy_cost(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("chainConfigTemplate")]
    fn chain_config_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("allDeployedContracts")]
    fn all_deployed_contracts(&self) -> UnorderedSetMapper<ManagedAddress>;
}
