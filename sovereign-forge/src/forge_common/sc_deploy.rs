use crate::err_msg;
use multiversx_sc::{
    imports::OptionalValue,
    sc_panic,
    types::{EgldPayment, ManagedAsyncCallResult, MultiValueEncoded},
};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::{ContractInfo, NativeToken, ScArray},
    PHASE_FOUR_ASYNC_CALL_GAS, PHASE_FOUR_CALLBACK_GAS, PHASE_ONE_ASYNC_CALL_GAS,
    PHASE_ONE_CALLBACK_GAS, PHASE_THREE_ASYNC_CALL_GAS, PHASE_THREE_CALLBACK_GAS,
    PHASE_TWO_ASYNC_CALL_GAS, PHASE_TWO_CALLBACK_GAS,
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
        sov_prefix: ManagedBuffer,
        native_token: NativeToken<Self::Api>,
        opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>,
    ) {
        let chain_id = self
            .sovereigns_mapper(&self.blockchain().get_caller())
            .get();

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_mvx_esdt_safe(sov_prefix, native_token, opt_config)
            .gas(PHASE_TWO_ASYNC_CALL_GAS)
            .egld(self.call_value().egld().clone())
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
}
