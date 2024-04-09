use crate::from_sovereign::token_mapping;
use bls_signature::BlsSignature;
use fee_market::subtract_fee::{FinalPayment, ProxyTrait as _};
use multiversx_sc::{hex_literal::hex, storage::StorageKey};
use transaction::{GasLimit, OperationData, TransferData};

multiversx_sc::imports!();

pub const ESDT_SYSTEM_SC_ADDRESS: [u8; 32] =
    hex!("000000000000000000010000000000000000000000000000000000000002ffff");
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
    + token_mapping::TokenMappingModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[endpoint(setMaxUserTxGasLimit)]
    fn set_max_user_tx_gas_limit(
        &self,
        new_value: GasLimit,
        opt_sig: OptionalValue<BlsSignature<Self::Api>>,
    ) {
        if !self.is_setup_phase_complete() {
            self.require_caller_initiator();
            self.max_user_tx_gas_limit().set(new_value);

            return;
        }

        let opt_signature = opt_sig.into_option();
        require!(opt_signature.is_some(), "Must provide signature");
        let signature = unsafe { opt_signature.unwrap_unchecked() };
        let mut signature_data = ManagedBuffer::new();
        let _ = new_value.dep_encode(&mut signature_data);

        self.multi_verify_signature(&signature_data, &signature);

        self.max_user_tx_gas_limit().set(new_value);
    }

    #[endpoint(setBurnAndMint)]
    fn set_burn_and_mint(
        &self,
        opt_signature: Option<BlsSignature<Self::Api>>,
        tokens: MultiValueEncoded<TokenIdentifier>,
    ) {
        if !self.is_setup_phase_complete() {
            self.require_caller_initiator();
            self.burn_tokens().extend(tokens);

            return;
        }

        let all_tokens = self.verify_items_signature(opt_signature, tokens);
        self.burn_tokens().extend(&all_tokens);
    }

    #[endpoint(removeBurnAndMint)]
    fn remove_burn_and_mint(
        &self,
        opt_signature: Option<BlsSignature<Self::Api>>,
        tokens: MultiValueEncoded<TokenIdentifier>,
    ) {
        if !self.is_setup_phase_complete() {
            self.require_caller_initiator();
            self.remove_items(&mut self.burn_tokens(), tokens);

            return;
        }

        let all_tokens = self.verify_items_signature(opt_signature, tokens);
        self.remove_items(&mut self.burn_tokens(), &all_tokens);
    }

    #[endpoint(addBannedEndpointNames)]
    fn add_banned_endpoint_names(
        &self,
        opt_signature: Option<BlsSignature<Self::Api>>,
        names: MultiValueEncoded<ManagedBuffer>,
    ) {
        if !self.is_setup_phase_complete() {
            self.require_caller_initiator();
            self.banned_endpoint_names().extend(names);

            return;
        }

        let all_names = self.verify_items_signature(opt_signature, names);
        self.banned_endpoint_names().extend(&all_names);
    }

    #[endpoint(removeBannedEndpointNames)]
    fn remove_banned_endpoint_names(
        &self,
        opt_signature: Option<BlsSignature<Self::Api>>,
        names: MultiValueEncoded<ManagedBuffer>,
    ) {
        if !self.is_setup_phase_complete() {
            self.require_caller_initiator();
            self.remove_items(&mut self.banned_endpoint_names(), names);

            return;
        }

        let all_names = self.verify_items_signature(opt_signature, names);
        self.remove_items(&mut self.banned_endpoint_names(), &all_names);
    }

    #[payable("*")]
    #[endpoint(depositBack)]
    fn deposit_back(&self, to: ManagedAddress) {
        require!(self.not_paused(), "Cannot create transaction while paused");

        let caller = self.blockchain().get_caller();
        require!(caller == ESDT_SYSTEM_SC_ADDRESS.into(), "Caller is invalid");

        let payments = self.call_value().all_esdt_transfers();
        self.send().direct_multi(&to, &payments);
    }

    fn check_and_extract_fee(
        &self,
    ) -> MultiValue2<OptionalValue<EsdtTokenPayment>, ManagedVec<EsdtTokenPayment>> {
        let mut payments = self.call_value().all_esdt_transfers().clone_value();
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

    /// Create an Elrond -> Sovereign transaction.
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

        let opt_transfer_data = self.process_transfer_data(opt_transfer_data);
        let own_sc_address = self.blockchain().get_sc_address();
        let mut total_tokens_for_fees = 0usize;
        let mut event_payments: MultiValueEncoded<
            MultiValue3<TokenIdentifier, u64, EsdtTokenData>,
        > = MultiValueEncoded::new();
        let mut refundable_payments: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> =
            ManagedVec::new();

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

            if self.is_sovereign_chain().get() {
                self.send().esdt_local_burn(
                    &payment.token_identifier,
                    payment.token_nonce,
                    &payment.amount,
                );

                event_payments.push(MultiValue3((
                    payment.token_identifier.clone(),
                    payment.token_nonce,
                    current_token_data.clone(),
                )));
            } else {
                let sov_token_id = self
                    .multiversx_to_sovereign_token_id(&payment.token_identifier)
                    .get();

                if !sov_token_id.is_valid_esdt_identifier() {
                    event_payments.push(MultiValue3((
                        payment.token_identifier,
                        payment.token_nonce,
                        current_token_data.clone(),
                    )));

                    continue;
                }

                self.send().esdt_local_burn(
                    &payment.token_identifier,
                    payment.token_nonce,
                    &payment.amount,
                );

                let mut sov_token_nonce = 0;

                if payment.token_nonce > 0 {
                    sov_token_nonce = self
                        .multiversx_esdt_token_info_mapper(
                            &payment.token_identifier,
                            &payment.token_nonce,
                        )
                        .take()
                        .token_nonce;

                    self.sovereign_esdt_token_info_mapper(&sov_token_id, &sov_token_nonce)
                        .take();
                }

                event_payments.push(MultiValue3((
                    sov_token_id,
                    sov_token_nonce,
                    current_token_data.clone(),
                )));
            }
        }

        let caller = self.blockchain().get_caller();

        match fees_payment {
            OptionalValue::Some(fee) => {
                let mut gas = 0;

                if let Some(transfer_data) = &opt_transfer_data {
                    gas = transfer_data.gas_limit;
                }

                let _: FinalPayment<Self::Api> = self
                    .fee_market_proxy(self.fee_market_address().get())
                    .subtract_fee(caller.clone(), total_tokens_for_fees, gas)
                    .with_esdt_transfer(fee)
                    .execute_on_dest_context();
            }
            OptionalValue::None => (),
        };

        // refund refundable_tokens
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

    #[proxy]
    fn fee_market_proxy(&self, sc_address: ManagedAddress) -> fee_market::Proxy<Self::Api>;

    #[storage_mapper("feeMarketAddress")]
    fn fee_market_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("maxUserTxGasLimit")]
    fn max_user_tx_gas_limit(&self) -> SingleValueMapper<GasLimit>;

    #[storage_mapper("burnTokens")]
    fn burn_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[storage_mapper("bannedEndpointNames")]
    fn banned_endpoint_names(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("feeEnabledFlag")]
    fn fee_enabled(&self) -> SingleValueMapper<bool>;
}
