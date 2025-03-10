use error_messages::{
    BANNED_ENDPOINT_NAME, DEPOSIT_OVER_MAX_AMOUNT, GAS_LIMIT_TOO_HIGH, NOTHING_TO_TRANSFER,
    TOKEN_ALREADY_REGISTERED, TOKEN_BLACKLISTED, TOO_MANY_TOKENS,
};
use proxies::fee_market_proxy::FeeMarketProxy;
use structs::{
    aliases::{EventPaymentTuple, ExtractedFeeResult, GasLimit, TxNonce},
    operation::TransferData,
};

use crate::MAX_TRANSFERS_PER_TX;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait DepositCommonModule:
    crate::storage::CrossChainStorage + crate::execute_common::ExecuteCommonModule + utils::UtilsModule
{
    fn match_fee_payment(
        &self,
        total_tokens_for_fees: usize,
        fees_payment: &OptionalValue<EsdtTokenPayment<Self::Api>>,
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

    fn check_and_extract_fee(&self) -> ExtractedFeeResult<Self::Api> {
        let payments = self.call_value().all_esdt_transfers().clone();

        require!(!payments.is_empty(), NOTHING_TO_TRANSFER);
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, TOO_MANY_TOKENS);
        let is_fee_enabled = self
            .external_fee_enabled(self.fee_market_address().get())
            .get();

        if !is_fee_enabled {
            return MultiValue2::from((OptionalValue::None, payments));
        };

        let (fee_payment, popped_payments) = self.pop_first_payment(payments.clone());

        MultiValue2::from((OptionalValue::Some(fee_payment), popped_payments))
    }

    fn burn_sovereign_token(&self, payment: &EsdtTokenPayment<Self::Api>) {
        self.tx()
            .to(ToSelf)
            .typed(system_proxy::UserBuiltinProxy)
            .esdt_local_burn(
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            )
            .sync_call();
    }

    fn get_event_payment_token_data(
        &self,
        current_sc_address: &ManagedAddress,
        payment: &EsdtTokenPayment<Self::Api>,
    ) -> EventPaymentTuple<Self::Api> {
        let mut current_token_data = self.blockchain().get_esdt_token_data(
            current_sc_address,
            &payment.token_identifier,
            payment.token_nonce,
        );
        current_token_data.amount = payment.amount.clone();

        MultiValue3::from((
            payment.token_identifier.clone(),
            payment.token_nonce,
            current_token_data,
        ))
    }

    fn is_above_max_amount(&self, token_id: &TokenIdentifier, amount: &BigUint) -> bool {
        let max_amount = self.max_bridged_amount(token_id).get();
        if max_amount > 0 {
            amount > &max_amount
        } else {
            false
        }
    }

    fn require_below_max_amount(&self, token_id: &TokenIdentifier, amount: &BigUint) {
        require!(
            !self.is_above_max_amount(token_id, amount),
            DEPOSIT_OVER_MAX_AMOUNT
        );
    }

    #[inline]
    fn refund_tokens(
        &self,
        caller: &ManagedAddress,
        refundable_payments: ManagedVec<EsdtTokenPayment<Self::Api>>,
    ) {
        self.tx()
            .to(caller)
            .multi_esdt(refundable_payments)
            .transfer_if_not_empty();
    }

    fn burn_mainchain_token(
        &self,
        payment: EsdtTokenPayment<Self::Api>,
        payment_token_type: &EsdtTokenType,
        sov_token_id: &TokenIdentifier<Self::Api>,
    ) -> u64 {
        self.tx()
            .to(ToSelf)
            .typed(system_proxy::UserBuiltinProxy)
            .esdt_local_burn(
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            )
            .sync_call();

        let mut sov_token_nonce = 0;

        if payment.token_nonce > 0 {
            sov_token_nonce = self
                .multiversx_to_sovereign_esdt_info_mapper(
                    &payment.token_identifier,
                    payment.token_nonce,
                )
                .get()
                .token_nonce;

            if self.is_nft(payment_token_type) {
                self.clear_mvx_to_sov_esdt_info_mapper(
                    &payment.token_identifier,
                    payment.token_nonce,
                );

                self.clear_sov_to_mvx_esdt_info_mapper(sov_token_id, sov_token_nonce);
            }
        }

        sov_token_nonce
    }

    #[inline]
    fn clear_mvx_to_sov_esdt_info_mapper(&self, id: &TokenIdentifier, nonce: u64) {
        self.multiversx_to_sovereign_esdt_info_mapper(id, nonce)
            .take();
    }

    #[inline]
    fn clear_sov_to_mvx_esdt_info_mapper(&self, id: &TokenIdentifier, nonce: u64) {
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
    fn require_sov_token_id_not_registered(&self, id: &TokenIdentifier) {
        require!(
            self.sovereign_to_multiversx_token_id_mapper(id).is_empty(),
            TOKEN_ALREADY_REGISTERED
        );
    }

    #[inline]
    fn require_token_not_on_blacklist(&self, token_id: &TokenIdentifier) {
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
    fn is_token_whitelisted(&self, token_id: &TokenIdentifier) -> bool {
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
