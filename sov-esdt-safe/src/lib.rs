#![no_std]
use error_messages::{CALLER_IS_NOT_TOKEN_OWNER, ISSUE_COST_NOT_COVERED};
use multiversx_sc::err_msg::TOKEN_IDENTIFIER_ESDT_EXPECTED;
#[allow(unused_imports)]
use multiversx_sc::imports::*;
use structs::configs::EsdtSafeConfig;
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

    #[payable("EGLD")]
    #[only_owner]
    #[endpoint(registerToken)]
    fn register_token(&self, token_identifier: EgldOrEsdtTokenIdentifier<Self::Api>) {
        require!(
            self.call_value().egld().clone() == ISSUE_COST,
            ISSUE_COST_NOT_COVERED
        );
        require!(token_identifier.is_esdt(), TOKEN_IDENTIFIER_ESDT_EXPECTED);
        let new_token_id = token_identifier.clone().unwrap_esdt();

        let token_properties = self
            .tx()
            .to(ESDTSystemSCAddress)
            .typed(ESDTSystemSCProxy)
            .get_token_properties(new_token_id)
            .returns(ReturnsResult)
            .sync_call();

        require!(
            self.blockchain().get_caller()
                == ManagedAddress::from_address(&token_properties.owner_address),
            CALLER_IS_NOT_TOKEN_OWNER
        );

        self.register_token_event(token_identifier, self.get_and_save_next_tx_id());
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
