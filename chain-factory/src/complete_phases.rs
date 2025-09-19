use crate::err_msg;
use multiversx_sc::imports::UserBuiltinProxy;
use multiversx_sc_modules::only_admin;
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, header_verifier_proxy::HeaderverifierProxy,
    mvx_esdt_safe_proxy::MvxEsdtSafeProxy, mvx_fee_market_proxy::MvxFeeMarketProxy,
};

#[multiversx_sc::module]
pub trait CompletePhasesModule: only_admin::OnlyAdminModule {
    #[only_admin]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(
        &self,
        chain_config_address: ManagedAddress,
        header_verifier_address: ManagedAddress,
        mvx_esdt_safe_address: ManagedAddress,
        fee_market_address: ManagedAddress,
    ) {
        self.tx()
            .to(&chain_config_address)
            .typed(ChainConfigContractProxy)
            .complete_setup_phase()
            .sync_call();

        self.tx()
            .to(&chain_config_address)
            .typed(UserBuiltinProxy)
            .change_owner_address(&header_verifier_address)
            .sync_call();

        self.tx()
            .to(&header_verifier_address)
            .typed(HeaderverifierProxy)
            .complete_setup_phase()
            .sync_call();

        self.tx()
            .to(&header_verifier_address)
            .typed(UserBuiltinProxy)
            .change_owner_address(&header_verifier_address)
            .sync_call();

        self.tx()
            .to(&mvx_esdt_safe_address)
            .typed(MvxEsdtSafeProxy)
            .complete_setup_phase()
            .sync_call();

        self.tx()
            .to(&mvx_esdt_safe_address)
            .typed(UserBuiltinProxy)
            .change_owner_address(&header_verifier_address)
            .sync_call();

        self.tx()
            .to(&fee_market_address)
            .typed(MvxFeeMarketProxy)
            .complete_setup_phase()
            .sync_call();

        self.tx()
            .to(&fee_market_address)
            .typed(UserBuiltinProxy)
            .change_owner_address(&header_verifier_address)
            .sync_call();
    }
}
