use crate::{
    err_msg,
    forge_common::{
        self,
        callbacks::{self, CallbackProxy},
    },
};

use error_messages::{
    CHAIN_CONFIG_ALREADY_DEPLOYED, DEPLOY_COST_NOT_ENOUGH, ESDT_SAFE_ALREADY_DEPLOYED,
    FEE_MARKET_ALREADY_DEPLOYED, HEADER_VERIFIER_ALREADY_DEPLOYED,
    SOVEREIGN_SETUP_PHASE_ALREADY_COMPLETED,
};
use multiversx_sc::{imports::OptionalValue, require, types::MultiValueEncoded};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::ScArray,
    COMPLETE_SETUP_PHASE_CALLBACK_GAS, COMPLETE_SETUP_PHASE_GAS,
};

#[multiversx_sc::module]
pub trait PhasesModule:
    forge_common::forge_utils::ForgeUtilsModule
    + forge_common::storage::StorageModule
    + forge_common::sc_deploy::ScDeployModule
    + custom_events::CustomEventsModule
    + common_utils::CommonUtilsModule
    + callbacks::ForgeCallbackModule
{
    #[payable("EGLD")]
    #[endpoint(deployPhaseOne)]
    fn deploy_phase_one(
        &self,
        opt_preferred_chain_id: Option<ManagedBuffer>,
        config: OptionalValue<SovereignConfig<Self::Api>>,
    ) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();
        let caller_shard_id = blockchain_api.get_shard_of_address(&caller);
        
        self.require_initialization_phase_complete(caller_shard_id);

        let call_value = self.call_value().egld().clone();
        require!(
            call_value == self.deploy_cost().get(),
            DEPLOY_COST_NOT_ENOUGH
        );

        let chain_id = self.generate_chain_id(opt_preferred_chain_id);

        let chain_factories_mapper = self.chain_factories(caller_shard_id);
        require!(
            !chain_factories_mapper.is_empty(),
            "There is no Chain-Factory address registered in shard {}",
            caller_shard_id
        );

        require!(
            !self.is_contract_deployed(&caller, ScArray::ChainConfig),
            CHAIN_CONFIG_ALREADY_DEPLOYED
        );

        self.deploy_chain_config(&caller, &chain_id, config);
        self.sovereigns_mapper(&caller).set(chain_id);
    }

    #[endpoint(deployPhaseTwo)]
    fn deploy_phase_two(&self, opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>) {
        let caller = self.blockchain().get_caller();
        let sov_prefix = self.sovereigns_mapper(&caller).get();

        self.require_phase_one_completed(&caller);
        require!(
            !self.is_contract_deployed(&caller, ScArray::ESDTSafe),
            ESDT_SAFE_ALREADY_DEPLOYED
        );

        self.deploy_mvx_esdt_safe(caller, sov_prefix, opt_config);
    }

    #[endpoint(deployPhaseThree)]
    fn deploy_phase_three(&self, fee: OptionalValue<FeeStruct<Self::Api>>) {
        let caller = self.blockchain().get_caller();

        self.require_phase_two_completed(&caller);
        require!(
            !self.is_contract_deployed(&caller, ScArray::FeeMarket),
            FEE_MARKET_ALREADY_DEPLOYED
        );

        let esdt_safe_address = self.get_contract_address(&caller, ScArray::ESDTSafe);

        self.deploy_fee_market(&caller, &esdt_safe_address, fee);
    }

    #[endpoint(deployPhaseFour)]
    fn deploy_phase_four(&self) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_three_completed(&caller);
        require!(
            !self.is_contract_deployed(&caller, ScArray::HeaderVerifier),
            HEADER_VERIFIER_ALREADY_DEPLOYED
        );

        let contract_addresses = MultiValueEncoded::from_iter(
            self.sovereign_deployed_contracts(&self.sovereigns_mapper(&caller).get())
                .iter(),
        );

        self.deploy_header_verifier(&caller, contract_addresses);
    }

    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self) {
        let caller = self.blockchain().get_caller();
        let sovereign_setup_phase_mapper =
            self.sovereign_setup_phase(&self.sovereigns_mapper(&caller).get());

        require!(
            sovereign_setup_phase_mapper.is_empty(),
            SOVEREIGN_SETUP_PHASE_ALREADY_COMPLETED
        );

        self.require_phase_four_completed(&caller);

        let chain_config_address = self.get_contract_address(&caller, ScArray::ChainConfig);
        let header_verifier_address = self.get_contract_address(&caller, ScArray::HeaderVerifier);
        let esdt_safe_address = self.get_contract_address(&caller, ScArray::ESDTSafe);
        let fee_market_address = self.get_contract_address(&caller, ScArray::FeeMarket);

        self.tx()
            .to(self.get_chain_factory_address(&caller))
            .typed(ChainFactoryContractProxy)
            .complete_setup_phase(
                chain_config_address,
                header_verifier_address,
                esdt_safe_address,
                fee_market_address,
            )
            .gas(COMPLETE_SETUP_PHASE_GAS)
            .callback(self.callbacks().setup_phase(&caller))
            .gas_for_callback(COMPLETE_SETUP_PHASE_CALLBACK_GAS)
            .register_promise();
    }
}
