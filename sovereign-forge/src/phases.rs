use crate::err_msg;
use core::ops::Deref;
use transaction::StakeMultiArg;

use multiversx_sc::{require, types::MultiValueEncoded};

use crate::common::{
    self,
    utils::{ContractInfo, ScArray},
};

const NUMBER_OF_SHARDS: u32 = 3;

#[multiversx_sc::module]
pub trait PhasesModule:
    common::utils::UtilsModule
    + common::storage::StorageModule
    + setup_phase::SetupPhaseModule
    + common::sc_deploy::ScDeployModule
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

        require!(
            !self.is_contract_deployed(&caller, ScArray::ChainConfig),
            "The Chain-Config contract is already deployed"
        );

        let chain_config_address = self.deploy_chain_config(
            min_validators,
            max_validators,
            min_stake,
            additional_stake_required,
        );

        let chain_factory_contract_info =
            ContractInfo::new(ScArray::ChainConfig, chain_config_address);

        self.sovereign_deployed_contracts(&chain_id)
            .insert(chain_factory_contract_info);
        self.sovereigns_mapper(&caller).set(chain_id);
    }

    #[endpoint(deployPhaseTwo)]
    fn deploy_phase_two(&self, bls_keys: MultiValueEncoded<ManagedBuffer>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_1_completed(&caller);

        let header_verifier_address = self.deploy_header_verifier(bls_keys);

        let header_verifier_contract_info =
            ContractInfo::new(ScArray::HeaderVerifier, header_verifier_address);

        self.sovereign_deployed_contracts(&self.sovereigns_mapper(&caller).get())
            .insert(header_verifier_contract_info);
    }
}
