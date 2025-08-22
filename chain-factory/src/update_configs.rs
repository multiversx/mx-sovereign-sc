use crate::err_msg;
use multiversx_sc::types::{MultiValueEncoded, TokenIdentifier};
use multiversx_sc_modules::only_admin;
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, fee_market_proxy::FeeMarketProxy,
    mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
};
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
};

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
            .update_esdt_safe_config_during_setup_phase(new_config)
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
            .update_sovereign_config_during_setup_phase(new_config)
            .sync_call();
    }

    #[only_admin]
    #[endpoint(setFee)]
    fn set_fee(&self, fee_market_address: ManagedAddress, new_fee: FeeStruct<Self::Api>) {
        self.tx()
            .to(fee_market_address)
            .typed(FeeMarketProxy)
            .set_fee_during_setup_phase(new_fee)
            .sync_call();
    }

    #[only_admin]
    #[endpoint(removeFee)]
    fn remove_fee(&self, fee_market_address: ManagedAddress, token_id: TokenIdentifier<Self::Api>) {
        self.tx()
            .to(fee_market_address)
            .typed(FeeMarketProxy)
            .remove_fee_during_setup_phase(token_id)
            .sync_call();
    }

    #[only_admin]
    #[endpoint(addUsersToWhitelistSetupPhase)]
    fn add_users_to_whitelist(
        &self,
        fee_market_address: ManagedAddress,
        users: MultiValueEncoded<ManagedAddress>,
    ) {
        self.tx()
            .to(fee_market_address)
            .typed(FeeMarketProxy)
            .add_users_to_whitelist_during_setup_phase(users)
            .sync_call();
    }

    #[only_admin]
    #[endpoint(removeUsersFromWhitelistSetupPhase)]
    fn remove_users_from_whitelist(
        &self,
        fee_market_address: ManagedAddress,
        users: MultiValueEncoded<ManagedAddress>,
    ) {
        self.tx()
            .to(fee_market_address)
            .typed(FeeMarketProxy)
            .remove_users_from_whitelist_during_setup_phase(users)
            .sync_call();
    }
}
