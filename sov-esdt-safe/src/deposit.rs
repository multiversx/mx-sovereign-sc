multiversx_sc::imports!();
use error_messages::ESDT_SAFE_STILL_PAUSED;
use structs::{
    aliases::{EventPaymentTuple, OptionalValueTransferDataTuple},
    operation::{OperationData, TransferData},
};

#[multiversx_sc::module]
pub trait DepositModule:
    multiversx_sc_modules::pause::PauseModule
    + utils::UtilsModule
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + cross_chain::storage::CrossChainStorage
    + cross_chain::events::EventsModule
{
    #[payable]
    #[endpoint]
    fn deposit(
        &self,
        to: ManagedAddress,
        opt_transfer_data: OptionalValueTransferDataTuple<Self::Api>,
    ) {
        require!(self.not_paused(), ESDT_SAFE_STILL_PAUSED);
        let option_transfer_data = TransferData::from_optional_value(&opt_transfer_data);

        if let Some(transfer_data) = &option_transfer_data {
            self.require_gas_limit_under_limit(transfer_data.gas_limit);
            self.require_endpoint_not_banned(&transfer_data.function);
        }

        let has_no_esdt_transfers = self.call_value().all_esdt_transfers().clone().is_empty();
        let deposit_without_transfers = has_no_esdt_transfers && opt_transfer_data.is_some();

        if deposit_without_transfers {
            let caller = self.blockchain().get_caller();
            let tx_nonce = self.get_and_save_next_tx_id();
            self.deposit_event(
                &to,
                &MultiValueEncoded::new(),
                OperationData::new(tx_nonce, caller, option_transfer_data),
            );
            return;
        }

        let (fees_payment, payments) = self.check_and_extract_fee().into_tuple();

        let mut total_tokens_for_fees = 0usize;
        let mut event_payments = MultiValueEncoded::new();
        let mut refundable_payments = ManagedVec::new();
        let current_sc_address = self.blockchain().get_sc_address();

        for payment in &payments {
            if let Some(event_payment) =
                self.process_payment(&current_sc_address, &payment, &mut refundable_payments)
            {
                total_tokens_for_fees += 1;
                event_payments.push(event_payment);
            }
        }

        let option_transfer_data = TransferData::from_optional_value(&opt_transfer_data);
        if let Some(transfer_data) = option_transfer_data.as_ref() {
            self.require_gas_limit_under_limit(transfer_data.gas_limit);
            self.require_endpoint_not_banned(&transfer_data.function);
        }

        self.match_fee_payment(total_tokens_for_fees, &fees_payment, &option_transfer_data);

        let caller = self.blockchain().get_caller();
        self.refund_tokens(&caller, refundable_payments);

        let tx_nonce = self.get_and_save_next_tx_id();
        self.deposit_event(
            &to,
            &event_payments,
            OperationData::new(tx_nonce, caller, option_transfer_data),
        );
    }

    fn process_payment(
        &self,
        current_sc_address: &ManagedAddress,
        payment: &EsdtTokenPayment<Self::Api>,
        refundable_payments: &mut ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>>,
    ) -> Option<EventPaymentTuple<Self::Api>> {
        self.require_below_max_amount(&payment.token_identifier, &payment.amount);
        self.require_token_not_on_blacklist(&payment.token_identifier);

        if !self.is_token_whitelist_empty() && !self.is_token_whitelisted(&payment.token_identifier)
        {
            refundable_payments.push(payment.clone());
            return None;
        }

        self.burn_sovereign_token(payment);
        Some(self.get_event_payment_token_data(current_sc_address, payment))
    }
}
