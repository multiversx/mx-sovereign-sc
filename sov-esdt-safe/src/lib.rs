#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;
use structs::configs::EsdtSafeConfig;

pub mod deposit;

#[multiversx_sc::contract]
pub trait SovEsdtSafe:
    deposit::DepositModule
    + cross_chain::LibCommon
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
        fee_market_address: ManagedAddress,
        opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>,
    ) {
        self.require_sc_address(&fee_market_address);
        self.fee_market_address().set(fee_market_address);

        self.esdt_safe_config().set(
            opt_config
                .into_option()
                .inspect(|config| self.require_esdt_config_valid(config))
                .unwrap_or_else(EsdtSafeConfig::default_config),
        );

        self.set_paused(true);
    }

    #[only_owner]
    #[endpoint(updateConfiguration)]
    fn update_configuration(&self, new_config: EsdtSafeConfig<Self::Api>) {
        self.require_esdt_config_valid(&new_config);
        self.esdt_safe_config().set(new_config);
    }

    #[only_owner]
    #[endpoint(setFeeMarketAddress)]
    fn set_fee_market_address(&self, fee_market_address: ManagedAddress) {
        self.require_sc_address(&fee_market_address);
        self.fee_market_address().set(fee_market_address);
    }

    #[only_owner]
    #[endpoint(setMaxBridgedAmount)]
    fn set_max_bridged_amount(&self, token_id: TokenIdentifier, max_amount: BigUint) {
        self.max_bridged_amount(&token_id).set(&max_amount);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
