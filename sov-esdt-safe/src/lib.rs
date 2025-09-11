#![no_std]
use error_messages::{EGLD_TOKEN_IDENTIFIER_EXPECTED, ISSUE_COST_NOT_COVERED, TOKEN_ID_NO_PREFIX};
use multiversx_sc::chain_core::EGLD_000000_TOKEN_IDENTIFIER;
#[allow(unused_imports)]
use multiversx_sc::imports::*;
use structs::{configs::EsdtSafeConfig, operation::OperationData};
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
    fn register_token(
        &self,
        token_id: EgldOrEsdtTokenIdentifier<Self::Api>,
        token_type: EsdtTokenType,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        token_decimals: usize,
    ) {
        let call_value = self.call_value().egld_or_single_esdt();
        require!(
            call_value.clone().token_identifier
                == EgldOrEsdtTokenIdentifier::from(EGLD_000000_TOKEN_IDENTIFIER),
            EGLD_TOKEN_IDENTIFIER_EXPECTED
        );
        require!(call_value.amount == ISSUE_COST, ISSUE_COST_NOT_COVERED);
        require!(self.has_prefix(&token_id), TOKEN_ID_NO_PREFIX);

        self.register_token_event(
            token_id,
            token_type,
            token_name,
            token_ticker,
            token_decimals,
            OperationData {
                op_nonce: self.get_and_save_next_tx_id(),
                op_sender: self.blockchain().get_caller(),
                opt_transfer_data: None,
            },
        );
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
