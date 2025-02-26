#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;
use operation::EsdtSafeConfig;

pub mod deposit;

#[multiversx_sc::contract]
pub trait SovEnshrineEsdtSafe:
    deposit::DepositModule
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + cross_chain::storage::CrossChainStorage
    + cross_chain::events::EventsModule
    + utils::UtilsModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[init]
    fn init(
        &self,
        token_handler_address: ManagedAddress,
        opt_config: Option<EsdtSafeConfig<Self::Api>>,
    ) {
        self.require_sc_address(&token_handler_address);
        self.set_paused(true);

        self.esdt_safe_config()
            .set(opt_config.unwrap_or_else(EsdtSafeConfig::default_config));
    }

    #[only_owner]
    #[endpoint(setFeeMarketAddress)]
    fn set_fee_market_address(&self, fee_market_address: ManagedAddress) {
        self.require_sc_address(&fee_market_address);

        self.fee_market_address().set(fee_market_address);
    }

    #[only_owner]
    #[endpoint(updateConfiguration)]
    fn update_configuration(&self, new_config: EsdtSafeConfig<Self::Api>) {
        self.esdt_safe_config().set(new_config);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
