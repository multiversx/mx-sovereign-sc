multiversx_sc::imports!();
use error_messages::ESDT_SAFE_STILL_PAUSED;
use structs::{
    aliases::OptionalValueTransferDataTuple,
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
    + max_bridged_amount_module::MaxBridgedAmountModule
{
    #[payable]
    #[endpoint]
    fn deposit(
        &self,
        to: ManagedAddress,
        opt_transfer_data: OptionalValueTransferDataTuple<Self::Api>,
    ) {
        require!(self.not_paused(), ESDT_SAFE_STILL_PAUSED);

        let (fees_payment, payments) = self.check_and_extract_fee().into_tuple();

        let mut total_tokens_for_fees = 0usize;
        let mut event_payments = MultiValueEncoded::new();
        let mut refundable_payments = ManagedVec::<Self::Api, _>::new();

        let own_sc_address = self.blockchain().get_sc_address();

        for payment in &payments {
            self.require_below_max_amount(&payment.token_identifier, &payment.amount);
            self.require_token_not_on_blacklist(&payment.token_identifier);

            if !self.is_token_whitelist_empty()
                && !self.is_token_whitelisted(&payment.token_identifier)
            {
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

            let mvx_to_sov_token_id_mapper =
                self.multiversx_to_sovereign_token_id_mapper(&payment.token_identifier);
            if !mvx_to_sov_token_id_mapper.is_empty() {
                let sov_token_id = mvx_to_sov_token_id_mapper.get();
                let sov_token_nonce = self.burn_mainchain_token(
                    payment.clone(),
                    &current_token_data.token_type,
                    &sov_token_id,
                );

                event_payments.push(MultiValue3::from((
                    sov_token_id,
                    sov_token_nonce,
                    current_token_data,
                )));
            } else {
                event_payments.push(MultiValue3::from((
                    payment.token_identifier.clone(),
                    payment.token_nonce,
                    current_token_data,
                )));
            }
        }

        let option_transfer_data = TransferData::from_optional_value(opt_transfer_data);

        if let Some(transfer_data) = option_transfer_data.as_ref() {
            self.require_gas_limit_under_limit(transfer_data.gas_limit);
            self.require_endpoint_not_banned(&transfer_data.function);
        }
        self.match_fee_payment(total_tokens_for_fees, &fees_payment, &option_transfer_data);

        // refund tokens
        let caller = self.blockchain().get_caller();
        self.refund_tokens(&caller, refundable_payments);

        let tx_nonce = self.get_and_save_next_tx_id();
        self.deposit_event(
            &to,
            &event_payments,
            OperationData::new(tx_nonce, caller, option_transfer_data),
        );
    }
}
