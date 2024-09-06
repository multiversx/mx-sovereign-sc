use chain_config::StakeMultiArg;

use multiversx_sc::imports::*;
use multiversx_sc_modules::only_admin;
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct ContractMapArgs<M: ManagedTypeApi> {
    id: ScArray,
    address: ManagedAddress<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
struct ChainInfo<M: ManagedTypeApi> {
    name: ManagedBuffer<M>,
    chain_id: ManagedBuffer<M>,
}

// TODO: Is fee market needed here?
#[derive(
    TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, PartialEq,
)]
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
pub trait FactoryModule: only_admin::OnlyAdminModule {
    // TODO: Check if contract was already deployed
    #[payable("EGLD")]
    #[endpoint(deploySovereignChainConfigContract)]
    fn deploy_sovereign_chain_config_contract(
        &self,
        min_validators: usize,
        max_validators: usize,
        min_stake: BigUint,
        chain_name: ManagedBuffer,
        additional_stake_required: MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) {
        let payment_amount = self.call_value().egld_value().clone_value();
        let deploy_cost = self.deploy_cost().get();
        require!(payment_amount == deploy_cost, "Invalid payment amount");

        let source_address = self.chain_config_template().get();
        let metadata =
            CodeMetadata::PAYABLE_BY_SC | CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE;
        let args = self.get_deploy_chain_config_args(
            &min_validators,
            &max_validators,
            &min_stake,
            &additional_stake_required,
        );

        let sc_address = self
            .tx()
            .raw_deploy()
            .from_source(source_address)
            .code_metadata(metadata)
            .arguments_raw(args)
            .returns(ReturnsNewManagedAddress)
            .sync_call();

        let caller = self.blockchain().get_caller();
        let mut set_admin_args = ManagedArgBuffer::new();
        set_admin_args.push_arg(&caller);

        self.tx()
            .to(&sc_address)
            .raw_call("add_admin")
            .arguments_raw(set_admin_args)
            .sync_call();

        let chain_id = self.generate_chain_id();

        self.all_deployed_contracts(chain_id.clone())
            .insert(ContractMapArgs {
                id: ScArray::ChainConfig,
                address: sc_address,
            });

        let chain_info = ChainInfo {
            name: chain_name,
            chain_id,
        };

        if self.current_chain_info().is_empty() {
            self.current_chain_info().set(chain_info);
        }

        self.add_admin(caller);
    }

    #[only_owner]
    #[endpoint(addContractsToMap)]
    fn add_contracts_to_map(
        &self,
        contracts_map: MultiValueEncoded<Self::Api, ContractMapArgs<Self::Api>>,
    ) {
        for contract in contracts_map {
            require!(
                self.contracts_map(contract.id.clone()).is_empty(),
                "There is already a SC address registered for that contract ID"
            );

            self.contracts_map(contract.id).set(contract.address);
        }
    }

    #[only_admin]
    #[endpoint(deployHeaderVerifier)]
    fn deploy_header_verifier(
        &self,
        chain_id: ManagedBuffer,
        bls_pub_keys: MultiValueEncoded<ManagedBuffer>,
    ) {
        let source_address = self.header_verifier_template().get();
        let metadata =
            CodeMetadata::PAYABLE_BY_SC | CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE;

        let mut args = ManagedArgBuffer::new();
        args.push_multi_arg(&bls_pub_keys);

        let header_verifier_address = self
            .tx()
            .raw_deploy()
            .from_source(source_address)
            .code_metadata(metadata)
            .arguments_raw(args)
            .returns(ReturnsNewManagedAddress)
            .sync_call();

        self.all_deployed_contracts(chain_id)
            .insert(ContractMapArgs {
                id: ScArray::SovereignHeaderVerifier,
                address: header_verifier_address,
            });
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

        let metadata =
            CodeMetadata::PAYABLE_BY_SC | CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE;

        let mut args = ManagedArgBuffer::new();
        args.push_arg(is_sovereign_chain);
        args.push_arg(token_handler_address);
        args.push_arg(opt_wegld_identifier);
        args.push_arg(opt_sov_token_prefix);

        let cross_chain_operations_address = self
            .tx()
            .raw_deploy()
            .from_source(source_address)
            .code_metadata(metadata)
            .arguments_raw(args)
            .returns(ReturnsNewManagedAddress)
            .sync_call();

        self.all_deployed_contracts(chain_id)
            .insert(ContractMapArgs {
                id: ScArray::SovereignCrossChainOperation,
                address: cross_chain_operations_address,
            });
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

        let metadata =
            CodeMetadata::PAYABLE_BY_SC | CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE;

        let mut args = ManagedArgBuffer::new();
        args.push_arg(esdt_safe_address);
        args.push_arg(price_aggregator_address);

        let fee_market_address = self
            .tx()
            .raw_deploy()
            .from_source(source_address)
            .code_metadata(metadata)
            .arguments_raw(args)
            .returns(ReturnsNewManagedAddress)
            .sync_call();

        self.all_deployed_contracts(chain_id)
            .insert(ContractMapArgs {
                id: ScArray::FeeMarket,
                address: fee_market_address,
            });
    }

    #[only_admin]
    #[endpoint(deployTokenHandler)]
    fn deploy_token_handler(&self, chain_id: ManagedBuffer) {
        let source_address = self.fee_market_template().get();

        let metadata =
            CodeMetadata::PAYABLE_BY_SC | CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE;

        let fee_market_address = self
            .tx()
            .raw_deploy()
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call();

        self.all_deployed_contracts(chain_id)
            .insert(ContractMapArgs {
                id: ScArray::TokenHandler,
                address: fee_market_address,
            });
    }
    fn get_deploy_chain_config_args(
        &self,
        min_validators: &usize,
        max_validators: &usize,
        min_stake: &BigUint,
        additional_stake_required: &MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) -> ManagedArgBuffer<Self::Api> {
        let mut args = ManagedArgBuffer::new();

        args.push_arg(min_validators);
        args.push_arg(max_validators);
        args.push_arg(min_stake);
        args.push_multi_arg(additional_stake_required);

        args
    }

    fn generate_chain_id(&self) -> ManagedBuffer {
        loop {
            let new_chain_id = self.generated_random_2_char_string();
            if !self.chain_ids().contains(&new_chain_id) {
                self.chain_ids().insert(new_chain_id.clone());

                return new_chain_id;
            }
        }
    }

    fn generated_random_2_char_string(&self) -> ManagedBuffer {
        let mut byte_array: [u8; 2] = [0; 2];
        let charset: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let mut rand = RandomnessSource::new();

        for i in 0..2 {
            let rand_index = rand.next_u8_in_range(0, charset.len() as u8) as usize;
            byte_array[i] = charset[rand_index];
        }

        ManagedBuffer::new_from_bytes(&byte_array)
    }

    #[view(getContractsMap)]
    #[storage_mapper("contractsMap")]
    fn contracts_map(&self, contract_name: ScArray) -> SingleValueMapper<ManagedAddress>;

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
        chain_id: ManagedBuffer,
    ) -> UnorderedSetMapper<ContractMapArgs<Self::Api>>;

    #[view(getCurrentChainInfo)]
    #[storage_mapper("currentChainInfo")]
    fn current_chain_info(&self) -> SingleValueMapper<ChainInfo<Self::Api>>;

    #[view(getAllChainIds)]
    #[storage_mapper("allChainIds")]
    fn chain_ids(&self) -> UnorderedSetMapper<ManagedBuffer>;
}
