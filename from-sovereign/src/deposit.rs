multiversx_sc::imports!();
use multiversx_sc::storage::StorageKey;
use operation::{
    aliases::{ExtractedFeeResult, GasLimit, OptionalValueTransferDataTuple},
    OperationData, TransferData,
};
use proxies::fee_market_proxy::FeeMarketProxy;

const MAX_TRANSFERS_PER_TX: usize = 10;

#[multiversx_sc::module]
pub trait DepositModule:
    multiversx_sc_modules::pause::PauseModule
    + utils::UtilsModule
    + cross_chain::CrossChainCommon
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
        require!(self.not_paused(), "Cannot create transaction while paused");

        let (fees_payment, payments) = self.check_and_extract_fee().into_tuple();
        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let mut total_tokens_for_fees = 0usize;
        let mut event_payments = MultiValueEncoded::new();
        let mut refundable_payments = ManagedVec::<Self::Api, _>::new();

        let own_sc_address = self.blockchain().get_sc_address();

        for payment in &payments {
            self.require_below_max_amount(&payment.token_identifier, &payment.amount);
            self.require_token_not_on_blacklist(&payment.token_identifier);
            // let is_token_whitelist_empty = self.token_whitelist().is_empty();
            // let is_token_whitelisted = self.token_whitelist().contains(&payment.token_identifier);

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

            self.tx()
                .to(ToSelf)
                .typed(system_proxy::UserBuiltinProxy)
                .esdt_local_burn(
                    &payment.token_identifier,
                    payment.token_nonce,
                    &payment.amount,
                )
                .sync_call();

            event_payments.push(MultiValue3::from((
                payment.token_identifier.clone(),
                payment.token_nonce,
                current_token_data,
            )));
        }

        let option_transfer_data = TransferData::from_optional_value(opt_transfer_data);

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
            &event_payments,
            OperationData::new(tx_nonce, caller, option_transfer_data),
        );
    }

    fn refund_tokens(
        &self,
        caller: &ManagedAddress,
        refundable_payments: ManagedVec<EsdtTokenPayment>,
    ) {
        for payment in refundable_payments {
            if payment.amount > 0 {
                self.tx().to(caller).payment(payment).transfer();
            }
        }
    }

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

        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let fee_market_address = self.fee_market_address().get();
        let fee_enabled_mapper = SingleValueMapper::new_from_address(
            fee_market_address.clone(),
            StorageKey::from("feeEnabledFlag"),
        )
        .get();

        let opt_transfer_data = if fee_enabled_mapper {
            OptionalValue::Some(self.pop_first_payment(payments.clone()).0)
        } else {
            OptionalValue::None
        };

        MultiValue2::from((opt_transfer_data, payments))
    }
}
