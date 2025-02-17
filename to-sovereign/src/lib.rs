#![no_std]

use multiversx_sc::imports::*;
use operation::EsdtSafeConfig;

pub mod deposit;

#[multiversx_sc::contract]
pub trait ToSovereign:
    deposit::DepositModule
    + cross_chain::CrossChainCommon
    + multiversx_sc_modules::pause::PauseModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + utils::UtilsModule
    + cross_chain::events::EventsModule
    + cross_chain::storage::CrossChainStorage
{
    #[init]
    fn init(&self, esdt_safe_config: EsdtSafeConfig<Self::Api>) {
        self.esdt_safe_config().set(esdt_safe_config);
    }

    #[endpoint(setFeeMarketAddress)]
    fn set_fee_market_address(&self, fee_market_address: ManagedAddress) {
        self.require_sc_address(&fee_market_address);
        self.fee_market_address().set(fee_market_address);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
