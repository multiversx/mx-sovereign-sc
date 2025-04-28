use crate::common;

use multiversx_sc::imports::*;
use structs::{
    aliases::OptionalValueTransferDataTuple,
    events::EventPayment,
    operation::{OperationData, TransferData},
};

const MAX_TRANSFERS_PER_TX: usize = 10;

#[multiversx_sc::module]
pub trait CreateTxModule:
    super::events::EventsModule
    + token_whitelist::TokenWhitelistModule
    + setup_phase::SetupPhaseModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + common::storage::CommonStorage
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + cross_chain::storage::CrossChainStorage
{
    #[payable("*")]
    #[endpoint]
    fn deposit(
        &self,
        to: ManagedAddress,
        optional_transfer_data: OptionalValueTransferDataTuple<Self::Api>,
    ) {
        require!(self.not_paused(), "Cannot create transaction while paused");

        let (fees_payment, payments) = self
            .check_and_extract_fee(optional_transfer_data.is_some())
            .into_tuple();

        let mut total_tokens_for_fees = 0usize;
        let mut event_payments = MultiValueEncoded::<Self::Api, EventPayment<Self::Api>>::new();
        let mut refundable_payments = ManagedVec::<Self::Api, _>::new();

        let own_sc_address = self.blockchain().get_sc_address();
        let is_sov_chain = self.is_sovereign_chain().get();

        for payment in &payments {
            self.require_below_max_amount(&payment.token_identifier, &payment.amount);
            self.require_token_not_blacklisted(&payment.token_identifier);
            let is_token_whitelist_empty = self.token_whitelist().is_empty();
            let is_token_whitelisted = self.token_whitelist().contains(&payment.token_identifier);

            if !is_token_whitelist_empty && !is_token_whitelisted {
                refundable_payments.push(payment.clone());

                continue;
            } else {
                total_tokens_for_fees += 1;
            }

            let mut current_token_data = self.blockchain().get_esdt_token_data(
                &own_sc_address,
                &payment.token_identifier,
                payment.token_nonce,
            );
            current_token_data.amount = payment.amount.clone();

            if is_sov_chain || self.has_prefix(&payment.token_identifier) {
                self.tx()
                    .to(ToSelf)
                    .typed(ESDTSystemSCProxy)
                    .burn(&payment.token_identifier, &payment.amount)
                    .transfer_execute();
            }

            let event_payment = EventPayment::new(
                payment.token_identifier.clone(),
                payment.token_nonce,
                current_token_data,
            );

            event_payments.push(event_payment);
        }

        let option_transfer_data = TransferData::from_optional_value(optional_transfer_data);

        if let Some(transfer_data) = option_transfer_data.as_ref() {
            self.require_gas_limit_under_limit(transfer_data.gas_limit);
            self.require_endpoint_not_banned(&transfer_data.function);
        }
        self.match_fee_payment(total_tokens_for_fees, &fees_payment, &option_transfer_data);

        // refund refundable_tokens
        let caller = self.blockchain().get_caller();
        self.refund_tokens(&caller, refundable_payments);

        let tx_nonce = self.get_and_save_next_tx_id();
        self.deposit_event(
            &to,
            &EventPayment::map_to_tuple_multi_value(event_payments),
            OperationData::new(tx_nonce, caller, option_transfer_data),
        );
    }
}
