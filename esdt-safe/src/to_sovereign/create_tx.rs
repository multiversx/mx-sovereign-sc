use crate::to_sovereign::events::DepositEvent;
use bls_signature::BlsSignature;
use fee_market::subtract_fee::{FinalPayment, ProxyTrait as _};
use multiversx_sc::storage::StorageKey;
use transaction::{GasLimit, StolenFromFrameworkEsdtTokenData};

multiversx_sc::imports!();

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
        let fee_market_address = self.fee_market_address().get();
        let mut payments = self.call_value().all_esdt_transfers().clone_value();
        let fee_enabled_mapper = SingleValueMapper::new_from_address(
            fee_market_address.clone(),
            StorageKey::from("feeEnabledFlag"),
        )
        .get();

        let fees_payment = if fee_enabled_mapper {
            OptionalValue::Some(self.pop_first_payment(&mut payments))
        } else {
            OptionalValue::None
        };

        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let (opt_gas_limit, opt_function, opt_args) = match &opt_transfer_data {
            OptionalValue::Some(transfer_data) => {
                let (gas_limit, function, args) = transfer_data.clone().into_tuple();
                let max_gas_limit = self.max_user_tx_gas_limit().get();
                require!(gas_limit <= max_gas_limit, "Gas limit too high");

                require!(
                    !self.banned_endpoint_names().contains(&function),
                    "Banned endpoint name"
                );

                (
                    OptionalValue::Some(gas_limit),
                    OptionalValue::Some(function),
                    OptionalValue::Some(args),
                )
            }
            OptionalValue::None => (
                OptionalValue::None,
                OptionalValue::None,
                OptionalValue::None,
            ),
        };

        let own_sc_address = self.blockchain().get_sc_address();
        let mut total_tokens_for_fees = 0usize;
        let mut event_payments = MultiValueEncoded::new();
        let burn_mapper = self.burn_tokens();
        let mut refundable_payments: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> =
            ManagedVec::new();

        for payment in &payments {
            self.require_below_max_amount(&payment.token_identifier, &payment.amount);
            self.require_token_not_blacklisted(&payment.token_identifier);

            if self.token_whitelist().len() > 0
                && !self.token_whitelist().contains(&payment.token_identifier)
            {
                refundable_payments.push(payment.clone());
                continue;
            } else {
                total_tokens_for_fees += 1;
            }

            let mut current_token_data = StolenFromFrameworkEsdtTokenData::default();
            if payment.token_nonce > 0 {
                current_token_data = self
                    .blockchain()
                    .get_esdt_token_data(
                        &own_sc_address,
                        &payment.token_identifier,
                        payment.token_nonce,
                    )
                    .into();
            }

            event_payments.push(MultiValue3((
                payment.token_identifier.clone(),
                payment.token_nonce,
                payment.amount.clone(), //use current_token_data
            )));

            if burn_mapper.contains(&payment.token_identifier) {
                self.send().esdt_local_burn(
                    &payment.token_identifier,
                    payment.token_nonce,
                    &payment.amount,
                );
            }
        }

        let caller = self.blockchain().get_caller();

        match fees_payment {
            OptionalValue::Some(fee) => {
                let _final_payments: FinalPayment<Self::Api> = self
                    .fee_market_proxy(fee_market_address)
                    .subtract_fee(caller.clone(), total_tokens_for_fees, opt_gas_limit.clone())
                    //why gas limit?
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
            DepositEvent {
                tx_nonce,
                opt_gas_limit: opt_gas_limit.into_option(),
                opt_function: opt_function.into_option(),
                opt_arguments: opt_args.into_option(),
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
