use crate::err_msg;
use multiversx_sc::{imports::OptionalValue, types::ReturnsResult};
use proxies::{chain_factory_proxy::ChainFactoryContractProxy, fee_market_proxy::FeeStruct};
use structs::configs::{EsdtSafeConfig, SovereignConfig};

#[multiversx_sc::module]
pub trait ScDeployModule: super::utils::UtilsModule + super::storage::StorageModule {
    #[inline]
    fn deploy_chain_config(&self, config: SovereignConfig<Self::Api>) -> ManagedAddress {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_sovereign_chain_config_contract(config)
            .returns(ReturnsResult)
            .sync_call()
    }

    #[inline]
    fn deploy_header_verifier(&self, chain_config_address: ManagedAddress) -> ManagedAddress {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_header_verifier(chain_config_address)
            .returns(ReturnsResult)
            .sync_call()
    }

    #[inline]
    fn deploy_mvx_esdt_safe(
        &self,
        header_verifier_address: &ManagedAddress,
        opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>,
    ) -> ManagedAddress {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_mvx_esdt_safe(header_verifier_address, opt_config)
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
