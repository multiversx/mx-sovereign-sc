use crate::err_msg;
use error_messages::SOVEREIGN_SETUP_PHASE_ALREADY_COMPLETED;
use multiversx_sc::{
    imports::{IgnoreValue, OptionalValue},
    require, sc_panic,
    types::{ManagedAsyncCallResult, MultiValueEncoded},
};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::{ContractInfo, ScArray},
    COMPLETE_SETUP_PHASE_GAS, PHASE_FOUR_ASYNC_CALL_GAS, PHASE_FOUR_CALLBACK_GAS,
    PHASE_ONE_ASYNC_CALL_GAS, PHASE_ONE_CALLBACK_GAS, PHASE_THREE_ASYNC_CALL_GAS,
    PHASE_THREE_CALLBACK_GAS, PHASE_TWO_ASYNC_CALL_GAS, PHASE_TWO_CALLBACK_GAS,
};

#[multiversx_sc::module]
pub trait ScDeployModule:
    super::forge_utils::ForgeUtilsModule
    + super::storage::StorageModule
    + common_utils::CommonUtilsModule
    + custom_events::CustomEventsModule
{
    #[inline]
    fn deploy_chain_config(
        &self,
        chain_id: &ManagedBuffer,
        config: OptionalValue<SovereignConfig<Self::Api>>,
    ) {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_sovereign_chain_config_contract(config)
            .gas(PHASE_ONE_ASYNC_CALL_GAS)
            .callback(
                self.callbacks()
                    .register_deployed_contract(chain_id, ScArray::ChainConfig),
            )
            .gas_for_callback(PHASE_ONE_CALLBACK_GAS)
            .register_promise();
    }

    #[inline]
    fn deploy_mvx_esdt_safe(
        &self,
        sovereign_owner: ManagedAddress,
        sov_prefix: ManagedBuffer,
        opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>,
    ) {
        let chain_id = self
            .sovereigns_mapper(&self.blockchain().get_caller())
            .get();

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_mvx_esdt_safe(sovereign_owner, sov_prefix, opt_config)
            .gas(PHASE_TWO_ASYNC_CALL_GAS)
            .callback(
                self.callbacks()
                    .register_deployed_contract(&chain_id, ScArray::ESDTSafe),
            )
            .gas_for_callback(PHASE_TWO_CALLBACK_GAS)
            .register_promise();
    }

    #[inline]
    fn deploy_fee_market(
        &self,
        esdt_safe_address: &ManagedAddress,
        fee: Option<FeeStruct<Self::Api>>,
    ) {
        let chain_id = self
            .sovereigns_mapper(&self.blockchain().get_caller())
            .get();

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_fee_market(esdt_safe_address, fee)
            .gas(PHASE_THREE_ASYNC_CALL_GAS)
            .callback(
                self.callbacks()
                    .register_deployed_contract(&chain_id, ScArray::FeeMarket),
            )
            .gas_for_callback(PHASE_THREE_CALLBACK_GAS)
            .register_promise();
    }

    #[inline]
    fn deploy_header_verifier(
        &self,
        sovereign_contract: MultiValueEncoded<ContractInfo<Self::Api>>,
    ) {
        let chain_id = self
            .sovereigns_mapper(&self.blockchain().get_caller())
            .get();

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_header_verifier(sovereign_contract)
            .gas(PHASE_FOUR_ASYNC_CALL_GAS)
            .callback(
                self.callbacks()
                    .register_deployed_contract(&chain_id, ScArray::HeaderVerifier),
            )
            .gas_for_callback(PHASE_FOUR_CALLBACK_GAS)
            .register_promise();
    }

    #[promises_callback]
    fn register_deployed_contract(
        &self,
        chain_id: &ManagedBuffer,
        sc_id: ScArray,
        #[call_result] result: ManagedAsyncCallResult<ManagedAddress>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(sc_address) => {
                let new_contract_info = ContractInfo::new(sc_id, sc_address);

                self.sovereign_deployed_contracts(chain_id)
                    .insert(new_contract_info);
            }
            ManagedAsyncCallResult::Err(call_err) => {
                sc_panic!(call_err.err_msg);
            }
        }
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
            .gas(COMPLETE_SETUP_PHASE_GAS)
            .callback(self.callbacks().setup_phase())
            .gas_for_callback(self.blockchain().get_gas_left())
            .register_promise();
    }

    #[promises_callback]
    fn setup_phase(&self, #[call_result] result: ManagedAsyncCallResult<IgnoreValue>) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                self.sovereign_setup_phase(
                    &self
                        .sovereigns_mapper(&self.blockchain().get_caller())
                        .get(),
                )
                .set(true);
            }
            ManagedAsyncCallResult::Err(result) => {
                sc_panic!(result.err_msg);
            }
        }
    }
}
