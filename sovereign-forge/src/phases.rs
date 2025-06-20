use crate::err_msg;
use core::ops::Deref;
use error_messages::{
    CHAIN_CONFIG_ALREADY_DEPLOYED, ESDT_SAFE_ALREADY_DEPLOYED, FEE_MARKET_ALREADY_DEPLOYED,
    HEADER_VERIFIER_ALREADY_DEPLOYED, SOVEREIGN_SETUP_PHASE_ALREADY_COMPLETED,
};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;

use multiversx_sc::{imports::OptionalValue, require, types::MultiValueEncoded};
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::{ContractInfo, ScArray},
};

use crate::common::{self};

#[multiversx_sc::module]
pub trait PhasesModule:
    common::utils::UtilsModule + common::storage::StorageModule + common::sc_deploy::ScDeployModule
{
    #[payable("EGLD")]
    #[endpoint(deployPhaseOne)]
    fn deploy_phase_one(
        &self,
        opt_preferred_chain_id: Option<ManagedBuffer>,
        config: OptionalValue<SovereignConfig<Self::Api>>,
    ) {
        self.require_initialization_phase_complete();

        let call_value = self.call_value().egld();
        self.require_correct_deploy_cost(call_value.deref());

        let chain_id = self.generate_chain_id(opt_preferred_chain_id);

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
            CHAIN_CONFIG_ALREADY_DEPLOYED
        );

        let chain_config_address = self.deploy_chain_config(config);

        let chain_config_contract_info =
            ContractInfo::new(ScArray::ChainConfig, chain_config_address);

        self.sovereign_deployed_contracts(&chain_id)
            .insert(chain_config_contract_info);
        self.sovereigns_mapper(&caller).set(chain_id);
    }

    #[endpoint(deployPhaseTwo)]
    fn deploy_phase_two(&self, opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>) {
        let caller = self.blockchain().get_caller();

        self.require_phase_one_completed(&caller);
        require!(
            !self.is_contract_deployed(&caller, ScArray::ESDTSafe),
            ESDT_SAFE_ALREADY_DEPLOYED
        );

        let esdt_safe_address = self.deploy_mvx_esdt_safe(opt_config);

        let esdt_safe_contract_info =
            ContractInfo::new(ScArray::ESDTSafe, esdt_safe_address.clone());

        self.sovereign_deployed_contracts(&self.sovereigns_mapper(&caller).get())
            .insert(esdt_safe_contract_info);
    }

    #[endpoint(deployPhaseThree)]
    fn deploy_phase_three(&self, fee: Option<FeeStruct<Self::Api>>) {
        let caller = self.blockchain().get_caller();

        self.require_phase_two_completed(&caller);
        require!(
            !self.is_contract_deployed(&caller, ScArray::FeeMarket),
            FEE_MARKET_ALREADY_DEPLOYED
        );

        let esdt_safe_address = self.get_contract_address(&caller, ScArray::ESDTSafe);

        let fee_market_address = self.deploy_fee_market(&esdt_safe_address, fee);

        let fee_market_contract_info = ContractInfo::new(ScArray::FeeMarket, fee_market_address);

        self.sovereign_deployed_contracts(&self.sovereigns_mapper(&caller).get())
            .insert(fee_market_contract_info);
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
        let header_verifier_address = self.deploy_header_verifier(contract_addresses);

        let header_verifier_contract_info =
            ContractInfo::new(ScArray::HeaderVerifier, header_verifier_address);

        self.sovereign_deployed_contracts(&self.sovereigns_mapper(&caller).get())
            .insert(header_verifier_contract_info);
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
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .complete_setup_phase(
                chain_config_address,
                header_verifier_address,
                esdt_safe_address,
                fee_market_address,
            )
            .sync_call();

        sovereign_setup_phase_mapper.set(true);
    }
}
