use chain_config::StakeMultiArg;

use multiversx_sc::imports::*;
use multiversx_sc_modules::only_admin;
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct ContractMapArgs<M: ManagedTypeApi> {
    pub chain_id: ManagedBuffer<M>,
    pub id: ScArray,
    pub address: ManagedAddress<M>,
}

// TODO: Is fee market needed here?
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, PartialEq)]
pub enum ScArray {
    ChainFactory,
    Controller,
    SovereignHeaderVerifier,
    SovereignCrossChainOperation,
    FeeMarket,
    TokenHandler,
    ChainConfig,
    Slashing,
}

#[multiversx_sc::module]
pub trait FactoryModule:
    only_admin::OnlyAdminModule
    + crate::common::storage::CommonStorage
    + crate::common::utils::UtilsModule
{
    // TODO: Check if contract was already deployed
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
        let args = self.get_deploy_chain_config_args(
            &min_validators,
            &max_validators,
            &min_stake,
            &caller,
            &additional_stake_required,
        );

        let chain_config_address = self.deploy_contract(source_address, args);

        let chain_id = self.generate_chain_id();
        self.set_deployed_contract_to_storage(
            &caller,
            chain_id,
            ScArray::ChainConfig,
            &chain_config_address,
        );

        self.add_admin(caller);
    }

    #[only_admin]
    #[endpoint(deployHeaderVerifier)]
    fn deploy_header_verifier(
        &self,
        chain_id: ManagedBuffer,
        bls_pub_keys: MultiValueEncoded<ManagedBuffer>,
    ) {
        let source_address = self.header_verifier_template().get();
        let mut args = ManagedArgBuffer::new();
        let caller = self.blockchain().get_caller();
        self.require_bls_keys_in_range(&caller, bls_pub_keys.len().into());
        args.push_multi_arg(&bls_pub_keys);

        let header_verifier_address = self.deploy_contract(source_address, args);

        self.set_deployed_contract_to_storage(
            &caller,
            chain_id,
            ScArray::SovereignHeaderVerifier,
            &header_verifier_address,
        );
    }

    #[only_admin]
    #[endpoint(deployCrossChainOperation)]
    fn deploy_cross_chain_operation(
        &self,
        chain_id: ManagedBuffer,
        is_sovereign_chain: bool,
        opt_wegld_identifier: Option<TokenIdentifier>,
        opt_sov_token_prefix: Option<ManagedBuffer>,
    ) {
        let source_address = self.cross_chain_operations_template().get();
        let token_handler_address = self.token_handler_template().get();

        let mut args = ManagedArgBuffer::new();
        args.push_arg(is_sovereign_chain);
        args.push_arg(token_handler_address);
        args.push_arg(opt_wegld_identifier);
        args.push_arg(opt_sov_token_prefix);

        let cross_chain_operations_address = self.deploy_contract(source_address, args);

        let caller = self.blockchain().get_caller();
        self.set_deployed_contract_to_storage(
            &caller,
            chain_id,
            ScArray::SovereignCrossChainOperation,
            &cross_chain_operations_address,
        );
    }

    #[only_admin]
    #[endpoint(deployFeeMarket)]
    fn deploy_fee_market(
        &self,
        chain_id: ManagedBuffer,
        esdt_safe_address: ManagedAddress,
        price_aggregator_address: ManagedAddress,
    ) {
        let source_address = self.fee_market_template().get();

        let mut args = ManagedArgBuffer::new();
        args.push_arg(esdt_safe_address);
        args.push_arg(price_aggregator_address);

        let fee_market_address = self.deploy_contract(source_address, args);

        let caller = self.blockchain().get_caller();
        self.set_deployed_contract_to_storage(
            &caller,
            chain_id,
            ScArray::FeeMarket,
            &fee_market_address,
        );
    }

    #[only_owner]
    #[endpoint(addContractsToMap)]
    fn add_contracts_to_map(
        &self,
        contracts_map: MultiValueEncoded<Self::Api, ContractMapArgs<Self::Api>>,
    ) {
        let caller = self.blockchain().get_caller();
        for contract in contracts_map {
            let contracts_mapper = self.all_deployed_contracts(&caller);

            require!(
                contracts_mapper.is_empty(),
                "There is already a SC address registered for that contract ID"
            );

            contracts_mapper.set(contract);
        }
    }

    fn deploy_contract(
        &self,
        source_address: ManagedAddress,
        args: ManagedArgBuffer<Self::Api>,
    ) -> ManagedAddress {
        let metadata = self.blockchain().get_code_metadata(&source_address);

        self.tx()
            .raw_deploy()
            .from_source(source_address)
            .code_metadata(metadata)
            .arguments_raw(args)
            .returns(ReturnsNewManagedAddress)
            .sync_call()
    }

    fn set_deployed_contract_to_storage(
        &self,
        caller: &ManagedAddress,
        chain_id: ManagedBuffer,
        contract_id: ScArray,
        contract_address: &ManagedAddress,
    ) {
        self.all_deployed_contracts(caller).set(ContractMapArgs {
            chain_id,
            id: contract_id,
            address: contract_address.clone(),
        });
    }

    fn get_deploy_chain_config_args(
        &self,
        min_validators: &usize,
        max_validators: &usize,
        min_stake: &BigUint,
        admin: &ManagedAddress,
        additional_stake_required: &MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) -> ManagedArgBuffer<Self::Api> {
        let mut args = ManagedArgBuffer::new();

        args.push_arg(min_validators);
        args.push_arg(max_validators);
        args.push_arg(min_stake);
        args.push_arg(admin);
        args.push_multi_arg(additional_stake_required);

        args
    }
}
