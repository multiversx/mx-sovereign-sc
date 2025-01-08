use crate::from_sovereign::token_mapping;
use multiversx_sc::storage::StorageKey;
use proxies::fee_market_proxy::FeeMarketProxy;
use operation::{
    aliases::{EventPaymentTuple, ExtractedFeeResult, GasLimit, OptionalValueTransferDataTuple},
    OperationData, TransferData,
};

multiversx_sc::imports!();

const MAX_TRANSFERS_PER_TX: usize = 10;

#[multiversx_sc::module]
pub trait CreateTxModule:
    super::events::EventsModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + token_whitelist::TokenWhitelistModule
    + setup_phase::SetupPhaseModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
    + token_mapping::TokenMappingModule
{
    #[payable("*")]
    #[endpoint]
    fn deposit(
        &self,
        to: ManagedAddress,
        optional_transfer_data: OptionalValueTransferDataTuple<Self::Api>,
    ) {
        require!(self.not_paused(), "Cannot create transaction while paused");

        let (fees_payment, payments) = self.check_and_extract_fee().into_tuple();
        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let mut total_tokens_for_fees = 0usize;
        let mut event_payments =
            MultiValueEncoded::<Self::Api, EventPaymentTuple<Self::Api>>::new();
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

            if is_sov_chain {
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
                    payment.token_identifier,
                    payment.token_nonce,
                    current_token_data,
                )));
            } else {
                let mvx_to_sov_token_id_mapper =
                    self.multiversx_to_sovereign_token_id_mapper(&payment.token_identifier);
                if !mvx_to_sov_token_id_mapper.is_empty() {
                    let sov_token_id = mvx_to_sov_token_id_mapper.get();
                    let sov_token_nonce = self.burn_mainchain_token(
                        payment,
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
                        payment.token_identifier,
                        payment.token_nonce,
                        current_token_data,
                    )));
                }
            }
        }

        let option_transfer_data = TransferData::from_optional_value(optional_transfer_data);

        if let Some(transfer_data) = option_transfer_data.as_ref() {
            self.require_gas_limit_under_limit(transfer_data.gas_limit);
            self.require_endpoint_not_banned(&transfer_data.function);
        }
        self.match_fee_payment(total_tokens_for_fees, &fees_payment, &option_transfer_data);

        // refund refundable_tokens
        let caller = self.blockchain().get_caller();
        self.refund_tokens(&caller, &refundable_payments);

        let tx_nonce = self.get_and_save_next_tx_id();
        self.deposit_event(
            &to,
            &event_payments,
            OperationData::new(tx_nonce, caller, option_transfer_data),
        );
    }

    fn check_and_extract_fee(&self) -> ExtractedFeeResult<Self::Api> {
        let mut payments = self.call_value().all_esdt_transfers().clone_value();

        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let fee_market_address = self.fee_market_address().get();
        let fee_enabled_mapper = SingleValueMapper::new_from_address(
            fee_market_address.clone(),
            StorageKey::from("feeEnabledFlag"),
        )
        .get();

        let opt_transfer_data = if fee_enabled_mapper {
            OptionalValue::Some(self.pop_first_payment(&mut payments))
        } else {
            OptionalValue::None
        };

        MultiValue2::from((opt_transfer_data, payments))
    }

    fn refund_tokens(
        &self,
        caller: &ManagedAddress,
        refundable_payments: &ManagedVec<EsdtTokenPayment>,
    ) {
        for payment in refundable_payments {
            if payment.amount > 0 {
                self.tx().to(caller).payment(&payment).transfer();
            }
        }
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

    fn match_fee_payment(
        &self,
        total_tokens_for_fees: usize,
        fees_payment: &OptionalValue<EsdtTokenPayment<Self::Api>>,
        opt_transfer_data: &Option<TransferData<Self::Api>>,
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
                    .payment(fee)
                    .sync_call();
            }
            OptionalValue::None => (),
        };
    }

    fn require_gas_limit_under_limit(&self, gas_limit: GasLimit) {
        let max_gas_limit = self.max_user_tx_gas_limit().get();
        require!(gas_limit <= max_gas_limit, "Gas limit too high");
    }

    fn require_endpoint_not_banned(&self, function: &ManagedBuffer) {
        require!(
            !self.banned_endpoint_names().contains(function),
            "Banned endpoint name"
        );
    }

    #[storage_mapper("feeMarketAddress")]
    fn fee_market_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("maxUserTxGasLimit")]
    fn max_user_tx_gas_limit(&self) -> SingleValueMapper<GasLimit>;

    #[storage_mapper("burnTokens")]
    fn burn_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[storage_mapper("bannedEndpointNames")]
    fn banned_endpoint_names(&self) -> UnorderedSetMapper<ManagedBuffer>;
}
