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

const NUMBER_OF_SHARDS: u32 = 3;

#[multiversx_sc::module]
pub trait PhasesModule:
    common::utils::UtilsModule + common::storage::StorageModule + setup_phase::SetupPhaseModule
{
    #[only_owner]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self) {
        if self.is_setup_phase_complete() {
            return;
        }

        for shard_id in 1..=NUMBER_OF_SHARDS {
            require!(
                !self.chain_factories(shard_id).is_empty(),
                "There is no Chain-Factory contract assigned for shard {}",
                shard_id
            );
            require!(
                !self.token_handlers(shard_id).is_empty(),
                "There is no Token-Handler contract assigned for shard {}",
                shard_id
            );
        }

        self.setup_phase_complete().set(true);
    }

    #[payable("EGLD")]
    #[endpoint(deployPhaseOne)]
    fn deploy_phase_one(
        &self,
        min_validators: u64,
        max_validators: u64,
        min_stake: BigUint,
        additional_stake_required: MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) {
        self.require_setup_complete();

        let call_value = self.call_value().egld_value();
        self.require_correct_deploy_cost(call_value.deref());

        let chain_id = self.generate_chain_id();
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();
        let caller_shard_id = blockchain_api.get_shard_of_address(&caller);

        let chain_factories_mapper = self.chain_factories(caller_shard_id);
        require!(
            !chain_factories_mapper.is_empty(),
            "There is no Chain-Factory address registered in shard {}",
            caller_shard_id
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

    #[endpoint(deployPhaseTwo)]
    fn deploy_phase_two(&self) {
        // check chain config was deployed && header was not
        // deploy header
        // update mapper
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

    fn deploy_header_verifier(
        &self,
        chain_factory_address: ManagedAddress,
        bls_keys: MultiValueEncoded<ManagedBuffer>,
    ) -> ManagedAddress {
        self.tx()
            .to(chain_factory_address)
            .typed(ChainFactoryContractProxy)
            .deploy_header_verifier(bls_keys)
            .returns(ReturnsResult)
            .sync_call()
    }

    fn deploy_esdt_safe(
        &self,
        chain_factory_address: ManagedAddress,
        is_sovereign_chain: bool,
    ) -> ManagedAddress {
        self.tx()
            .to(chain_factory_address)
            .typed(ChainFactoryContractProxy)
            .deploy_esdt_safe(is_sovereign_chain)
            .returns(ReturnsResult)
            .sync_call()
    }
}
