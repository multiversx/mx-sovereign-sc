use crate::err_msg;
use multiversx_sc::{
    imports::OptionalValue,
    types::{MultiValueEncoded, ReturnsResult},
};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::ContractInfo,
};

#[multiversx_sc::module]
pub trait ScDeployModule: super::utils::UtilsModule + super::storage::StorageModule {
    #[inline]
    fn deploy_chain_config(
        &self,
        config: OptionalValue<SovereignConfig<Self::Api>>,
    ) -> ManagedAddress {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_sovereign_chain_config_contract(config)
            .returns(ReturnsResult)
            .sync_call()
    }

    #[inline]
    fn deploy_header_verifier(
        &self,
        sovereign_contract: MultiValueEncoded<ContractInfo<Self::Api>>,
    ) -> ManagedAddress {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_header_verifier(sovereign_contract)
            .returns(ReturnsResult)
            .sync_call()
    }

    #[inline]
    fn deploy_mvx_esdt_safe(
        &self,
        opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>,
    ) -> ManagedAddress {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_mvx_esdt_safe(opt_config)
            .returns(ReturnsResult)
            .sync_call()
    }

    #[inline]
    fn deploy_fee_market(
        &self,
        esdt_safe_address: &ManagedAddress,
        fee: Option<FeeStruct<Self::Api>>,
    ) -> ManagedAddress {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_fee_market(esdt_safe_address, fee)
            .returns(ReturnsResult)
            .sync_call()
    }
}
