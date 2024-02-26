use bls_signature::BlsSignature;
use fee_market::{__endpoints_3__::is_fee_enabled, enable_fee::ProxyTrait as _, subtract_fee::{FinalPayment, ProxyTrait as _}};
use multiversx_sc::api::HandleConstraints;
use transaction::{GasLimit, StolenFromFrameworkEsdtTokenData, Transaction, TransferData};

use crate::to_sovereign::events::DepositEvent;

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
        opt_transfer_data: OptionalValue<MultiValue3<GasLimit, ManagedBuffer, ManagedVec<ManagedBuffer>>>
    ) {
        require!(self.not_paused(), "Cannot create transaction while paused");
        let fee_market_address = self.fee_market_address().get();
        let mut payments = self.call_value().all_esdt_transfers().clone_value();

        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let opt_gas_limit = match &opt_transfer_data {
            OptionalValue::Some(transfer_data) => {
                let (gas_limit, function, args) = transfer_data.into_tuple();
                let max_gas_limit = self.max_user_tx_gas_limit().get();
                require!(
                    gas_limit <= max_gas_limit,
                    "Gas limit too high"
                );

                require!(
                    !self
                        .banned_endpoint_names()
                        .contains(&function),
                    "Banned endpoint name"
                );

                OptionalValue::Some(gas_limit)
            }
            OptionalValue::None => OptionalValue::None,
        };

        let own_sc_address = self.blockchain().get_sc_address();
        let mut all_token_data = ManagedVec::new();
        let mut total_tokens_for_fees = 0usize;
        let mut event_payments = MultiValueEncoded::new();
        let burn_mapper = self.burn_tokens();
        for payment in &payments {
            self.require_below_max_amount(&payment.token_identifier, &payment.amount);
            self.require_token_not_blacklisted(&payment.token_identifier);

            if !self.token_whitelist().contains(&payment.token_identifier) {
                total_tokens_for_fees += 1;
            }

            if payment.token_nonce > 0 {
                let current_token_data = self.blockchain().get_esdt_token_data(
                    &own_sc_address,
                    &payment.token_identifier,
                    payment.token_nonce,
                );
                all_token_data.push(current_token_data.into());
            } else {
                all_token_data.push(StolenFromFrameworkEsdtTokenData::default());
            }
            
            event_payments.push(
                MultiValue3 ((
                    payment.token_identifier.clone(),
                    payment.token_nonce,
                    payment.amount.clone()    
                ))
            );

            if burn_mapper.contains(&payment.token_identifier) {
                self.send().esdt_local_burn(
                    &payment.token_identifier,
                    payment.token_nonce,
                    &payment.amount,
                );
            }
        }

        let caller = self.blockchain().get_caller();
        let is_fee_enabled: bool = self.fee_market_proxy(fee_market_address.clone()).is_fee_enabled().execute_on_dest_context();
        let token_id = TokenIdentifier::from(ManagedBuffer::new());
        let mut fees_payment = EsdtTokenPayment::new(token_id, 0 as u64, BigUint::from(0u64));

        if is_fee_enabled {
            fees_payment = self.pop_first_payment(&mut payments);
        }
        // match is_fee_enabled {
        //     true => {
        //         fees_payment = self.pop_first_payment(&mut payments);
        //     }
        //
        //     false => {
        //         _
        //     }
        // }

        let final_payments: FinalPayment<Self::Api> = self
            .fee_market_proxy(fee_market_address)
            .subtract_fee(caller.clone(), total_tokens_for_fees, opt_gas_limit)
            .with_esdt_transfer(fees_payment)
            .execute_on_dest_context();

        self.send()
            .direct_non_zero_esdt_payment(&caller, &final_payments.remaining_tokens);

        let block_nonce = self.blockchain().get_block_nonce();
        let tx_nonce = self.get_and_save_next_tx_id();

        self.deposit_event(
            &to,
            &event_payments,
            DepositEvent::from(tx_nonce, &opt_transfer_data),
        );

        let tx = Transaction {
            block_nonce,
            nonce: tx_nonce,
            from: caller,
            to,
            tokens: payments,
            token_data: all_token_data,
            opt_transfer_data: opt_transfer_data.into_option(),
            is_refund_tx: false,
        };
        let _ = self.add_to_batch(tx);
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
}
