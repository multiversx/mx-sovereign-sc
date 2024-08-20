use crate::common;
use fee_market::fee_market_proxy;
use transaction::{GasLimit, OperationData, TransferData};

use multiversx_sc::imports::*;

const MAX_TRANSFERS_PER_TX: usize = 10;

#[multiversx_sc::module]
pub trait CreateTxModule:
    super::events::EventsModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + token_whitelist::TokenWhitelistModule
    + bls_signature::BlsSignatureModule
    + setup_phase::SetupPhaseModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + common::storage::CommonStorage
{
    #[payable("*")]
    #[endpoint]
    fn deposit(
        &self,
        to: ManagedAddress,
        opt_transfer_data: OptionalValue<
            MultiValue3<GasLimit, ManagedBuffer, ManagedVec<ManagedBuffer>>,
        >,
    ) {
        require!(self.not_paused(), "Cannot create transaction while paused");

        let (fees_payment, payments) = self.check_and_extract_fee().into_tuple();
        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let mut total_tokens_for_fees = 0usize;
        let mut event_payments = MultiValueEncoded::new();
        let mut refundable_payments = ManagedVec::<Self::Api, _>::new();

        let opt_transfer_data = self.process_transfer_data(opt_transfer_data);
        let own_sc_address = self.blockchain().get_sc_address();
        let is_sov_chain = self.is_sovereign_chain().get();

        for payment in &payments {
            self.require_below_max_amount(&payment.token_identifier, &payment.amount);
            self.require_token_not_blacklisted(&payment.token_identifier);

            if !self.token_whitelist().is_empty()
                && !self.token_whitelist().contains(&payment.token_identifier)
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

            if is_sov_chain || self.has_prefix(&payment.token_identifier) {
                self.send().esdt_local_burn(
                    &payment.token_identifier,
                    payment.token_nonce,
                    &payment.amount,
                );
            }

            event_payments.push(
                (
                    payment.token_identifier.clone(),
                    payment.token_nonce,
                    current_token_data.clone(),
                )
                    .into(),
            );
        }

        self.match_fee_payment(total_tokens_for_fees, &fees_payment, &opt_transfer_data);

        // refund refundable_tokens
        let caller = self.blockchain().get_caller();
        for payment in &refundable_payments {
            self.send().direct_non_zero_esdt_payment(&caller, &payment);
        }

        let tx_nonce = self.get_and_save_next_tx_id();
        self.deposit_event(
            &to,
            &event_payments,
            OperationData {
                op_nonce: tx_nonce,
                op_sender: caller,
                opt_transfer_data,
            },
        );
    }

    fn check_and_extract_fee(
        &self,
    ) -> MultiValue2<OptionalValue<EsdtTokenPayment>, ManagedVec<EsdtTokenPayment>> {
        let mut payments = self.call_value().all_esdt_transfers().clone_value();

        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let fee_market_address = self.fee_market_address().get();
        let fee_enabled = self.external_fee_enabled(fee_market_address).get();
        let opt_transfer_data = if fee_enabled {
            OptionalValue::Some(self.pop_first_payment(&mut payments))
        } else {
            OptionalValue::None
        };

        MultiValue2::from((opt_transfer_data, payments))
    }

    fn process_transfer_data(
        &self,
        opt_transfer_data: OptionalValue<
            MultiValue3<GasLimit, ManagedBuffer, ManagedVec<ManagedBuffer>>,
        >,
    ) -> Option<TransferData<Self::Api>> {
        match &opt_transfer_data {
            OptionalValue::Some(transfer_data) => {
                let (gas_limit, function, args) = transfer_data.clone().into_tuple();
                let max_gas_limit = self.max_user_tx_gas_limit().get();

                require!(gas_limit <= max_gas_limit, "Gas limit too high");

                require!(
                    !self.banned_endpoint_names().contains(&function),
                    "Banned endpoint name"
                );

                Some(TransferData {
                    gas_limit,
                    function,
                    args,
                })
            }
            OptionalValue::None => None,
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
                    .typed(fee_market_proxy::FeeMarketProxy)
                    .subtract_fee(caller, total_tokens_for_fees, OptionalValue::Some(gas))
                    .payment(fee.clone())
                    .async_call_and_exit();
            }
            OptionalValue::None => (),
        };
    }

    #[storage_mapper("feeMarketAddress")]
    fn fee_market_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("maxUserTxGasLimit")]
    fn max_user_tx_gas_limit(&self) -> SingleValueMapper<GasLimit>;

    #[storage_mapper("burnTokens")]
    fn burn_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[storage_mapper("bannedEndpointNames")]
    fn banned_endpoint_names(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper_from_address("feeEnabledFlag")]
    fn external_fee_enabled(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<bool, ManagedAddress>;
}
