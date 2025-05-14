use crate::err_msg;
use multiversx_sc_modules::only_admin;
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
};
use structs::configs::{EsdtSafeConfig, SovereignConfig};

#[multiversx_sc::module]
pub trait UpdateConfigsModule: only_admin::OnlyAdminModule {
    #[only_admin]
    #[endpoint(updateEsdtSafeConfig)]
    fn update_esdt_safe_config(
        &self,
        esdt_safe_address: ManagedAddress,
        new_config: EsdtSafeConfig<Self::Api>,
    ) {
        self.tx()
            .to(esdt_safe_address)
            .typed(MvxEsdtSafeProxy)
            .update_configuration(new_config)
            .sync_call();
    }

    #[only_admin]
    #[endpoint(updateSovereignConfig)]
    fn update_sovereign_config(
        &self,
        chain_config_address: ManagedAddress,
        new_config: SovereignConfig<Self::Api>,
    ) {
        self.tx()
            .to(chain_config_address)
            .typed(ChainConfigContractProxy)
            .update_config(new_config)
            .sync_call();
    }
}
