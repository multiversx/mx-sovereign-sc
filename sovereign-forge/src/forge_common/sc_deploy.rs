use crate::{
    err_msg,
    forge_common::callbacks::{self, CallbackProxy},
};
use multiversx_sc::{imports::OptionalValue, types::MultiValueEncoded};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::{ContractInfo, ScArray},
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
    + callbacks::ForgeCallbackModule
{
    #[inline]
    fn deploy_chain_config(
        &self,
        sovereign_owner: &ManagedAddress,
        chain_id: &ManagedBuffer,
        config: OptionalValue<SovereignConfig<Self::Api>>,
    ) {
        self.tx()
            .to(self.get_chain_factory_address(sovereign_owner))
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
        let chain_id = self.sovereigns_mapper(&sovereign_owner).get();

        self.tx()
            .to(self.get_chain_factory_address(&sovereign_owner))
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
        sovereign_owner: &ManagedAddress,
        esdt_safe_address: &ManagedAddress,
        fee: OptionalValue<FeeStruct<Self::Api>>,
    ) {
        let chain_id = self.sovereigns_mapper(sovereign_owner).get();

        self.tx()
            .to(self.get_chain_factory_address(sovereign_owner))
            .typed(ChainFactoryContractProxy)
            .deploy_fee_market(esdt_safe_address, fee.into_option())
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
        sovereign_owner: &ManagedAddress,
        sovereign_contract: MultiValueEncoded<ContractInfo<Self::Api>>,
    ) {
        let chain_id = self.sovereigns_mapper(sovereign_owner).get();

        self.tx()
            .to(self.get_chain_factory_address(sovereign_owner))
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
}
