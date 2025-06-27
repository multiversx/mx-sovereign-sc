use crate::err_msg;
use multiversx_sc::{
    imports::OptionalValue,
    sc_panic,
    types::{ManagedAsyncCallResult, MultiValueEncoded},
};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::{ContractInfo, ScArray},
};

#[multiversx_sc::module]
pub trait ScDeployModule: super::utils::UtilsModule + super::storage::StorageModule {
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
            .gas(self.blockchain().get_gas_left() / 2)
            .callback(
                self.callbacks()
                    .register_deployed_contract(chain_id, ScArray::ChainConfig),
            )
            .gas_for_callback(15_000_000)
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
            .gas(self.blockchain().get_gas_left() / 2)
            .callback(
                self.callbacks()
                    .register_deployed_contract(&chain_id, ScArray::HeaderVerifier),
            )
            .gas_for_callback(15_000_000)
            .register_promise();
    }

    #[inline]
    fn deploy_mvx_esdt_safe(&self, opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>) {
        let chain_id = self
            .sovereigns_mapper(&self.blockchain().get_caller())
            .get();

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_mvx_esdt_safe(opt_config)
            .gas(self.blockchain().get_gas_left() / 2)
            .callback(
                self.callbacks()
                    .register_deployed_contract(&chain_id, ScArray::ESDTSafe),
            )
            .gas_for_callback(15_000_000)
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
            .gas(self.blockchain().get_gas_left() / 2)
            .callback(
                self.callbacks()
                    .register_deployed_contract(&chain_id, ScArray::FeeMarket),
            )
            .gas_for_callback(15_000_000)
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
