#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;
use operation::EsdtSafeConfig;

pub mod deposit;

#[multiversx_sc::contract]
pub trait SovEsdtSafe:
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
    fn init(&self, header_verifier_address: ManagedAddress, config: EsdtSafeConfig<Self::Api>) {
        self.require_sc_address(&header_verifier_address);
        self.header_verifier_address().set(&header_verifier_address);
        self.esdt_safe_config().set(config);
    }

    #[only_owner]
    #[endpoint(setFeeMarketAddress)]
    fn set_fee_market_address(&self, fee_market_address: ManagedAddress) {
        self.require_sc_address(&fee_market_address);
        self.fee_market_address().set(fee_market_address);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
