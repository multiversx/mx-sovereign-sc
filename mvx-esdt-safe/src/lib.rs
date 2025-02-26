#![no_std]

use multiversx_sc::imports::*;
use operation::EsdtSafeConfig;

pub mod deposit;
pub mod execute;
pub mod register_token;

#[multiversx_sc::contract]
pub trait MvxEsdtSafe:
    deposit::DepositModule
    + execute::ExecuteModule
    + register_token::RegisterTokenModule
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::events::EventsModule
    + cross_chain::storage::CrossChainStorage
    + cross_chain::execute_common::ExecuteCommonModule
    + multiversx_sc_modules::pause::PauseModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + utils::UtilsModule
{
    #[init]
    fn init(
        &self,
        header_verifier_address: ManagedAddress,
        esdt_safe_config: EsdtSafeConfig<Self::Api>,
    ) {
        self.require_sc_address(&header_verifier_address);
        self.header_verifier_address().set(&header_verifier_address);
        self.esdt_safe_config().set(esdt_safe_config);
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
