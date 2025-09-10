#![no_std]
use error_messages::{EGLD_TOKEN_IDENTIFIER_EXPECTED, ISSUE_COST_NOT_COVERED, TOKEN_ID_NO_PREFIX};
#[allow(unused_imports)]
use multiversx_sc::imports::*;
use multiversx_sc::{
    chain_core::EGLD_000000_TOKEN_IDENTIFIER, err_msg::TOKEN_IDENTIFIER_ESDT_EXPECTED,
};
use structs::{configs::EsdtSafeConfig, RegisterTokenStruct};
pub const ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD

pub mod deposit;

#[multiversx_sc::contract]
pub trait SovEsdtSafe:
    deposit::DepositModule
    + cross_chain::LibCommon
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + cross_chain::storage::CrossChainStorage
    + custom_events::CustomEventsModule
    + common_utils::CommonUtilsModule
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

        let new_config = match opt_config {
            OptionalValue::Some(cfg) => {
                if let Some(error_message) = self.is_esdt_safe_config_valid(&cfg) {
                    sc_panic!(error_message);
                }
                cfg
            }
            OptionalValue::None => EsdtSafeConfig::default_config(),
        };

        self.esdt_safe_config().set(new_config);

        self.set_paused(true);
    }

    #[payable]
    #[only_owner]
    #[endpoint(registerToken)]
    fn register_token(&self, new_token: RegisterTokenStruct<Self::Api>) {
        let call_value = self.call_value().single_esdt();
        require!(
            call_value.clone().token_identifier
                == TokenIdentifier::from(EGLD_000000_TOKEN_IDENTIFIER),
            EGLD_TOKEN_IDENTIFIER_EXPECTED
        );
        require!(call_value.amount == ISSUE_COST, ISSUE_COST_NOT_COVERED);
        require!(self.has_prefix(&new_token.token_id), TOKEN_ID_NO_PREFIX);

        self.register_token_event(new_token, self.get_and_save_next_tx_id());
    }

    #[only_owner]
    #[endpoint(updateConfiguration)]
    fn update_configuration(&self, new_config: EsdtSafeConfig<Self::Api>) {
        if let Some(error_message) = self.is_esdt_safe_config_valid(&new_config) {
            sc_panic!(error_message);
        }

        self.esdt_safe_config().set(new_config);
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
