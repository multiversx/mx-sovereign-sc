use error_messages::{
    BANNED_ENDPOINT_NAME, DEPOSIT_OVER_MAX_AMOUNT, ESDT_SAFE_STILL_PAUSED, GAS_LIMIT_TOO_HIGH,
    NOTHING_TO_TRANSFER, TOKEN_BLACKLISTED, TOO_MANY_TOKENS,
};
use proxies::fee_market_proxy::FeeMarketProxy;
use structs::{
    aliases::{
        EventPaymentTuple, ExtractedFeeResult, GasLimit, OptionalValueTransferDataTuple, TxNonce,
    },
    operation::{OperationData, TransferData},
};

use crate::MAX_TRANSFERS_PER_TX;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait DepositCommonModule:
    crate::storage::CrossChainStorage
    + crate::execute_common::ExecuteCommonModule
    + common_utils::CommonUtilsModule
    + custom_events::CustomEventsModule
    + multiversx_sc_modules::pause::PauseModule
{
    fn deposit_common<F>(
        &self,
        to: ManagedAddress,
        opt_transfer_data: OptionalValueTransferDataTuple<Self::Api>,
        process_payment: F,
    ) where
        F: Fn(&EgldOrEsdtTokenPayment<Self::Api>) -> EventPaymentTuple<Self::Api>,
    {
        require!(self.not_paused(), ESDT_SAFE_STILL_PAUSED);

        let option_transfer_data = TransferData::from_optional_value(opt_transfer_data.clone());

        if let Some(transfer_data) = option_transfer_data.as_ref() {
            self.require_gas_limit_under_limit(transfer_data.gas_limit);
            self.require_endpoint_not_banned(&transfer_data.function);
        }

        let (fees_payment, payments) = self
            .check_and_extract_fee(opt_transfer_data.is_some())
            .into_tuple();

        let mut total_tokens_for_fees = 0usize;
        let mut event_payments = MultiValueEncoded::new();
        let mut refundable_payments = ManagedVec::<Self::Api, _>::new();

        for payment in &payments {
            let token_identifier = payment.token_identifier.clone();
            self.require_below_max_amount(&token_identifier, &payment.amount);
            self.require_token_not_on_blacklist(&token_identifier);

            if !self.is_token_whitelist_empty() && !self.is_token_whitelisted(&token_identifier) {
                refundable_payments.push(payment.clone());
                continue;
            }
            total_tokens_for_fees += 1;

            let processed_payment = process_payment(&payment);

            event_payments.push(processed_payment);
        }

        self.match_fee_payment(total_tokens_for_fees, &fees_payment, &option_transfer_data);

        let caller = self.blockchain().get_caller();
        self.refund_tokens(&caller, refundable_payments);

        let tx_nonce = self.get_and_save_next_tx_id();

        if payments.is_empty() {
            self.sc_call_event(
                &to,
                OperationData::new(tx_nonce, caller, option_transfer_data),
            );

            return;
        }

        self.deposit_event(
            &to,
            &event_payments,
            OperationData::new(tx_nonce, caller, option_transfer_data),
        );
    }

    fn match_fee_payment(
        &self,
        total_tokens_for_fees: usize,
        fees_payment: &OptionalValue<EgldOrEsdtTokenPayment<Self::Api>>,
        opt_transfer_data: &Option<TransferData<<Self as ContractBase>::Api>>,
    ) {
        match fees_payment {
            OptionalValue::Some(fee) => {
                let mut gas: GasLimit = 0;

                if let Some(transfer_data) = opt_transfer_data {
                    gas = transfer_data.gas_limit;
                }

                let fee_market_address = self.fee_market_address().get();
                let caller = self.blockchain().get_caller();

                self.tx()
                    .to(fee_market_address)
                    .typed(FeeMarketProxy)
                    .subtract_fee(caller, total_tokens_for_fees, OptionalValue::Some(gas))
                    .payment(fee.clone())
                    .sync_call();
            }
            OptionalValue::None => (),
        };
    }

    fn check_and_extract_fee(&self, has_transfer_data: bool) -> ExtractedFeeResult<Self::Api> {
        let payments = self.call_value().all_transfers().clone();
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, TOO_MANY_TOKENS);

        let fee_enabled = self
            .external_fee_enabled(self.fee_market_address().get())
            .get();

        if fee_enabled {
            return self.pop_first_payment(payments);
        } else {
            if payments.is_empty() {
                require!(has_transfer_data, NOTHING_TO_TRANSFER);
            }

            MultiValue2::from((OptionalValue::None, payments))
        }
    }

    fn burn_sovereign_token(&self, payment: &EgldOrEsdtTokenPayment<Self::Api>) {
        self.tx()
            .to(ToSelf)
            .typed(system_proxy::UserBuiltinProxy)
            .esdt_local_burn(
                payment.token_identifier.clone().unwrap_esdt(),
                payment.token_nonce,
                &payment.amount,
            )
            .sync_call();
    }

    fn get_event_payment_token_data(
        &self,
        current_sc_address: &ManagedAddress,
        payment: &EgldOrEsdtTokenPayment<Self::Api>,
    ) -> EventPaymentTuple<Self::Api> {
        let mut current_token_data = self.blockchain().get_esdt_token_data(
            current_sc_address,
            &payment.token_identifier.clone().unwrap_esdt(),
            payment.token_nonce,
        );
        current_token_data.amount = payment.amount.clone();

        MultiValue3::from((
            payment.token_identifier.as_managed_buffer().clone(),
            payment.token_nonce,
            current_token_data,
        ))
    }

    fn is_above_max_amount(&self, token_id: &EgldOrEsdtTokenIdentifier, amount: &BigUint) -> bool {
        self.esdt_safe_config()
            .get()
            .max_bridged_token_amounts
            .iter()
            .any(|m| m.token_id == *token_id && amount > &m.amount)
    }

    fn require_below_max_amount(&self, token_id: &EgldOrEsdtTokenIdentifier, amount: &BigUint) {
        require!(
            !self.is_above_max_amount(token_id, amount),
            DEPOSIT_OVER_MAX_AMOUNT
        );
    }

    #[inline]
    fn refund_tokens(
        &self,
        caller: &ManagedAddress,
        refundable_payments: ManagedVec<EgldOrEsdtTokenPayment<Self::Api>>,
    ) {
        self.tx()
            .to(caller)
            .payment(refundable_payments)
            .transfer_if_not_empty();
    }

    fn burn_mainchain_token(
        &self,
        token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        token_nonce: u64,
        amount: &BigUint,
        payment_token_type: &EsdtTokenType,
        sov_token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
    ) -> u64 {
        self.tx()
            .to(ToSelf)
            .typed(system_proxy::UserBuiltinProxy)
            .esdt_local_burn(token_id.clone().unwrap_esdt(), token_nonce, amount)
            .sync_call();

        let mut sov_token_nonce = 0;

        if token_nonce > 0 {
            sov_token_nonce = self
                .multiversx_to_sovereign_esdt_info_mapper(token_id, token_nonce)
                .get()
                .token_nonce;

            if self.is_nft(payment_token_type) {
                self.clear_mvx_to_sov_esdt_info_mapper(token_id, token_nonce);

                self.clear_sov_to_mvx_esdt_info_mapper(sov_token_id, sov_token_nonce);
            }
        }

        sov_token_nonce
    }

    #[inline]
    fn clear_mvx_to_sov_esdt_info_mapper(
        &self,
        id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        nonce: u64,
    ) {
        self.multiversx_to_sovereign_esdt_info_mapper(id, nonce)
            .take();
    }

    #[inline]
    fn clear_sov_to_mvx_esdt_info_mapper(
        &self,
        id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        nonce: u64,
    ) {
        self.sovereign_to_multiversx_esdt_info_mapper(id, nonce)
            .take();
    }

    #[inline]
    fn get_and_save_next_tx_id(&self) -> TxNonce {
        self.last_tx_nonce().update(|last_tx_nonce| {
            *last_tx_nonce += 1;
            *last_tx_nonce
        })
    }

    #[inline]
    fn is_sov_token_id_registered(&self, id: &EgldOrEsdtTokenIdentifier<Self::Api>) -> bool {
        !self.sovereign_to_multiversx_token_id_mapper(id).is_empty()
    }

    #[inline]
    fn require_token_not_on_blacklist(&self, token_id: &EgldOrEsdtTokenIdentifier<Self::Api>) {
        require!(
            !self
                .esdt_safe_config()
                .get()
                .token_blacklist
                .contains(token_id),
            TOKEN_BLACKLISTED
        );
    }

    #[inline]
    fn is_token_whitelist_empty(&self) -> bool {
        self.esdt_safe_config().get().token_whitelist.is_empty()
    }

    #[inline]
    fn require_endpoint_not_banned(&self, function: &ManagedBuffer) {
        require!(
            !self
                .esdt_safe_config()
                .get()
                .banned_endpoints
                .contains(function),
            BANNED_ENDPOINT_NAME
        );
    }

    #[inline]
    fn is_token_whitelisted(&self, token_id: &EgldOrEsdtTokenIdentifier<Self::Api>) -> bool {
        self.esdt_safe_config()
            .get()
            .token_whitelist
            .contains(token_id)
    }

    #[inline]
    fn require_gas_limit_under_limit(&self, gas_limit: GasLimit) {
        require!(
            gas_limit <= self.esdt_safe_config().get().max_tx_gas_limit,
            GAS_LIMIT_TOO_HIGH
        );
    }
}
