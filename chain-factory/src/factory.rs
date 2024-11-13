use chain_config::StakeMultiArg;

use multiversx_sc::imports::*;
use multiversx_sc_modules::only_admin;
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, header_verifier_proxy::HeaderverifierProxy,
};
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct ContractInfo<M: ManagedTypeApi> {
    pub id: ScArray,
    pub address: ManagedAddress<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct ChainContractsMap<M: ManagedTypeApi> {
    pub chain_id: ManagedBuffer<M>,
    pub contracts_info: ManagedVec<M, ContractInfo<M>>,
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

        let metadata = self.blockchain().get_code_metadata(&source_address);
        let chain_config_address = self
            .tx()
            .typed(ChainConfigContractProxy)
            .init(
                min_validators,
                max_validators,
                min_stake,
                &caller,
                additional_stake_required,
            )
            .gas(60_000_000)
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call();

        let chain_id = self.generate_chain_id();
        self.set_deployed_contract_to_storage(
            &caller,
            chain_id,
            ScArray::ChainConfig,
            &chain_config_address,
        );
    }

    #[only_admin]
    #[endpoint(deployHeaderVerifier)]
    fn deploy_header_verifier(&self, bls_pub_keys: MultiValueEncoded<ManagedBuffer>) {
        let caller = self.blockchain().get_caller();

        require!(
            self.is_chain_deployed(&caller),
            "The current caller has not deployed and Sovereign Chain"
        );

        let chain_id = self.all_deployed_contracts(&caller).get().chain_id;
        let source_address = self.header_verifier_template().get();

        self.require_bls_keys_in_range(&caller, &chain_id, bls_pub_keys.len().into());

        let metadata = self.blockchain().get_code_metadata(&source_address);
        let header_verifier_address = self
            .tx()
            .typed(HeaderverifierProxy)
            .init(bls_pub_keys)
            .gas(60_000_000)
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call();

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
        is_sovereign_chain: bool,
        opt_wegld_identifier: Option<TokenIdentifier>,
        opt_sov_token_prefix: Option<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();
        require!(
            self.is_chain_deployed(&caller),
            "The current caller has not deployed and Sovereign Chain"
        );

        let chain_id = self.all_deployed_contracts(&caller).get().chain_id;
        let source_address = self.cross_chain_operations_template().get();
        let token_handler_address = self.token_handler_template().get();

        let mut args = ManagedArgBuffer::new();
        args.push_arg(is_sovereign_chain);
        args.push_arg(token_handler_address);
        args.push_arg(opt_wegld_identifier);
        args.push_arg(opt_sov_token_prefix);

        let cross_chain_operations_address = self.deploy_contract(source_address, args);

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
        esdt_safe_address: ManagedAddress,
        price_aggregator_address: ManagedAddress,
    ) {
        let caller = self.blockchain().get_caller();
        require!(
            self.is_chain_deployed(&caller),
            "The current caller has not deployed and Sovereign Chain"
        );

        let chain_id = self.all_deployed_contracts(&caller).get().chain_id;
        let source_address = self.fee_market_template().get();

        let mut args = ManagedArgBuffer::new();
        args.push_arg(esdt_safe_address);
        args.push_arg(price_aggregator_address);

        let fee_market_address = self.deploy_contract(source_address, args);

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
        chain_id: ManagedBuffer,
        contracts_info: MultiValueEncoded<Self::Api, ContractInfo<Self::Api>>,
    ) {
        let caller = self.blockchain().get_caller();
        let contracts_mapper = self.all_deployed_contracts(&caller);
        let mut contracts_info_for_registration = ManagedVec::new();

        for contract_info in contracts_info {
            let is_contract_registered =
                self.is_contract_registered(&caller, &chain_id, &contract_info.id);

            if is_contract_registered {
                continue;
            }
            contracts_info_for_registration.push(contract_info);
        }

        contracts_mapper.set(ChainContractsMap {
            chain_id,
            contracts_info: contracts_info_for_registration,
        });
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
        let contract_info = ContractInfo {
            id: contract_id,
            address: contract_address.clone(),
        };

        let mut contracts_info = ManagedVec::new();
        contracts_info.push(contract_info);

        self.all_deployed_contracts(caller).set(ChainContractsMap {
            chain_id,
            contracts_info,
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
