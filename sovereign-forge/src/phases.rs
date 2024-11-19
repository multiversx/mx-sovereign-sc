use core::ops::Deref;
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use transaction::StakeMultiArg;

use multiversx_sc::{
    require,
    types::{ManagedVec, MultiValueEncoded, ReturnsResult},
};

use crate::{
    common::{
        self,
        utils::{ChainContractsMap, ContractInfo, ScArray},
    },
    err_msg,
};

#[multiversx_sc::module]
pub trait PhasesModule: common::utils::UtilsModule + common::storage::StorageModule {
    #[payable("EGLD")]
    #[endpoint(deployPhaseOne)]
    fn deploy_phase_one(
        &self,
        min_validators: u64,
        max_validators: u64,
        min_stake: BigUint,
        additional_stake_required: MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) {
        let call_value = self.call_value().egld_value();
        self.require_correct_deploy_cost(call_value.deref());

        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();
        let caller_shard_id = blockchain_api.get_shard_of_address(&caller);

        let chain_id = self.generate_chain_id();

        let chain_factories_mapper = self.chain_factories(caller_shard_id);
        require!(
            !chain_factories_mapper.is_empty(),
            "There is no Chain-Factory address registered"
        );

        let chain_factory_address = chain_factories_mapper.get();

        let chain_config_address = self.deploy_chain_config(
            chain_factory_address,
            min_validators,
            max_validators,
            min_stake,
            additional_stake_required,
        );

        let sovereigns_mapper = self.sovereigns_mapper(&caller);
        require!(
            sovereigns_mapper.is_empty(),
            "There is already a deployed Sovereign Chain for this user"
        );

        let chain_factory_contract_info =
            ContractInfo::new(ScArray::ChainConfig, chain_config_address);

        let mut contracts_info = ManagedVec::new();
        contracts_info.push(chain_factory_contract_info);

        let chain_contracts_map = ChainContractsMap::new(chain_id, contracts_info);

        sovereigns_mapper.set(chain_contracts_map);
    }

    fn deploy_chain_config(
        &self,
        chain_factory_address: ManagedAddress,
        min_validators: u64,
        max_validators: u64,
        min_stake: BigUint,
        additional_stake_required: MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) -> ManagedAddress {
        self.tx()
            .to(chain_factory_address)
            .typed(ChainFactoryContractProxy)
            .deploy_sovereign_chain_config_contract(
                min_validators,
                max_validators,
                min_stake,
                additional_stake_required,
            )
            .returns(ReturnsResult)
            .sync_call()
    }
}
