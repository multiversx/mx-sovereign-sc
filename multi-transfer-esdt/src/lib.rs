#![no_std]

multiversx_sc::imports!();

use transaction::{
    PaymentsVec, StolenFromFrameworkEsdtTokenData, Transaction, TxBatchSplitInFields,
};

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = u64::MAX;
const NFT_AMOUNT: u32 = 1;

#[multiversx_sc::contract]
pub trait MultiTransferEsdt:
    tx_batch_module::TxBatchModule + max_bridged_amount_module::MaxBridgedAmountModule
{
    #[init]
    fn init(&self, opt_wrapping_contract_address: OptionalValue<ManagedAddress>) {
        self.max_tx_batch_size()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        self.set_wrapping_contract_address(opt_wrapping_contract_address);

        // batch ID 0 is considered invalid
        self.first_batch_id().set_if_empty(1);
        self.last_batch_id().set_if_empty(1);
    }

    #[only_owner]
    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        batch_id: u64,
        transfers: MultiValueEncoded<Transaction<Self::Api>>,
    ) {
        let mut valid_payments_list = ManagedVec::new();
        let mut valid_dest_addresses_list = ManagedVec::new();
        let mut refund_tx_list = ManagedVec::new();

        let own_sc_address = self.blockchain().get_sc_address();
        let sc_shard = self.blockchain().get_shard_of_address(&own_sc_address);

        for sov_tx in transfers {
            let mut refund_tokens_for_user = ManagedVec::new();
            let mut tokens_to_send = ManagedVec::new();
            let mut sent_token_data = ManagedVec::new();

            for (token, opt_token_data) in sov_tx.tokens.iter().zip(sov_tx.token_data.iter()) {
                let must_refund =
                    self.check_must_refund(&token, &sov_tx.to, batch_id, sov_tx.nonce, sc_shard);

                if must_refund {
                    refund_tokens_for_user.push(token);
                } else {
                    tokens_to_send.push(token);
                    sent_token_data.push(opt_token_data);
                }
            }

            if !refund_tokens_for_user.is_empty() {
                let refund_tx = self.convert_to_refund_tx(sov_tx.clone(), refund_tokens_for_user);
                refund_tx_list.push(refund_tx);
            }

            if tokens_to_send.is_empty() {
                continue;
            }

            let user_tokens_to_send = self.mint_tokens(tokens_to_send, sent_token_data);

            // emit event before the actual transfer so we don't have to save the tx_nonces as well
            self.transfer_performed_event(batch_id, sov_tx.nonce);

            valid_dest_addresses_list.push(sov_tx.to);
            valid_payments_list.push(user_tokens_to_send);
        }

        let payments_after_wrapping = self.wrap_tokens(valid_payments_list);
        self.distribute_payments(valid_dest_addresses_list, payments_after_wrapping);

        self.add_multiple_tx_to_batch(&refund_tx_list);
    }

    #[only_owner]
    #[endpoint(getAndClearFirstRefundBatch)]
    fn get_and_clear_first_refund_batch(&self) -> OptionalValue<TxBatchSplitInFields<Self::Api>> {
        let opt_current_batch = self.get_first_batch_any_status();
        if matches!(opt_current_batch, OptionalValue::Some(_)) {
            let first_batch_id = self.first_batch_id().get();
            let mut first_batch = self.pending_batches(first_batch_id);

            self.clear_first_batch(&mut first_batch);
        }

        opt_current_batch
    }

    #[only_owner]
    #[endpoint(setWrappingContractAddress)]
    fn set_wrapping_contract_address(&self, opt_new_address: OptionalValue<ManagedAddress>) {
        match opt_new_address {
            OptionalValue::Some(sc_addr) => {
                require!(
                    self.blockchain().is_smart_contract(&sc_addr),
                    "Invalid unwrapping contract address"
                );

                self.wrapping_contract_address().set(&sc_addr);
            }
            OptionalValue::None => self.wrapping_contract_address().clear(),
        }
    }

    // private

    fn check_must_refund(
        &self,
        token: &EsdtTokenPayment,
        dest: &ManagedAddress,
        batch_id: u64,
        tx_nonce: u64,
        sc_shard: u32,
    ) -> bool {
        if token.token_nonce == 0 {
            if !self.is_local_role_set(&token.token_identifier, &EsdtLocalRole::Mint) {
                self.transfer_failed_invalid_token(batch_id, tx_nonce);

                return true;
            }
        } else {
            if !self.is_local_role_set(&token.token_identifier, &EsdtLocalRole::NftCreate) {
                self.transfer_failed_invalid_token(batch_id, tx_nonce);

                return true;
            } else if token.amount > NFT_AMOUNT
                && !self.is_local_role_set(&token.token_identifier, &EsdtLocalRole::NftAddQuantity)
            {
                self.transfer_failed_invalid_token(batch_id, tx_nonce);

                return true;
            }
        }

        if self.is_above_max_amount(&token.token_identifier, &token.amount) {
            self.transfer_over_max_amount(batch_id, tx_nonce);

            return true;
        }

        if self.is_account_same_shard_frozen(sc_shard, dest, &token.token_identifier) {
            self.transfer_failed_frozen_destination_account(batch_id, tx_nonce);

            return true;
        }

        false
    }

    fn convert_to_refund_tx(
        &self,
        sov_tx: Transaction<Self::Api>,
        tokens_to_refund: PaymentsVec<Self::Api>,
    ) -> Transaction<Self::Api> {
        Transaction {
            block_nonce: self.blockchain().get_block_nonce(),
            nonce: sov_tx.nonce,
            from: sov_tx.from,
            to: sov_tx.to,
            tokens: tokens_to_refund,
            token_data: ManagedVec::new(),
            opt_transfer_data: None,
            is_refund_tx: true,
        }
    }

    fn mint_tokens(
        &self,
        payments: PaymentsVec<Self::Api>,
        all_token_data: ManagedVec<Option<StolenFromFrameworkEsdtTokenData<Self::Api>>>,
    ) -> PaymentsVec<Self::Api> {
        let mut output_payments = PaymentsVec::new();
        for (payment, opt_token_data) in payments.iter().zip(all_token_data.iter()) {
            if payment.token_nonce == 0 {
                self.send()
                    .esdt_local_mint(&payment.token_identifier, 0, &payment.amount);

                output_payments.push(EsdtTokenPayment::new(
                    payment.token_identifier,
                    0,
                    payment.amount,
                ));

                continue;
            }

            require!(opt_token_data.is_some(), "Invalid token data");

            let token_data = unsafe { opt_token_data.unwrap_unchecked() };
            let token_nonce = self.send().esdt_nft_create(
                &payment.token_identifier,
                &payment.amount,
                &token_data.name,
                &token_data.royalties,
                &token_data.hash,
                &token_data.attributes,
                &token_data.uris,
            );
            output_payments.push(EsdtTokenPayment::new(
                payment.token_identifier,
                token_nonce,
                payment.amount,
            ));
        }

        output_payments
    }

    fn is_local_role_set(&self, token_id: &TokenIdentifier, role: &EsdtLocalRole) -> bool {
        let roles = self.blockchain().get_esdt_local_roles(token_id);

        roles.has_role(role)
    }

    fn is_account_same_shard_frozen(
        &self,
        sc_shard: u32,
        dest_address: &ManagedAddress,
        token_id: &TokenIdentifier,
    ) -> bool {
        let dest_shard = self.blockchain().get_shard_of_address(dest_address);
        if sc_shard != dest_shard {
            return false;
        }

        let token_data = self
            .blockchain()
            .get_esdt_token_data(dest_address, token_id, 0);
        token_data.frozen
    }

    fn wrap_tokens(
        &self,
        payments: ManagedVec<PaymentsVec<Self::Api>>,
    ) -> ManagedVec<PaymentsVec<Self::Api>> {
        if self.wrapping_contract_address().is_empty() {
            return payments;
        }

        // TODO: Think about making a single call here
        let mut output_payments = ManagedVec::new();
        for user_payments in &payments {
            let wrapped_payments: PaymentsVec<Self::Api> = self
                .get_wrapping_contract_proxy_instance()
                .wrap_tokens()
                .with_multi_token_transfer(user_payments)
                .execute_on_dest_context();

            output_payments.push(wrapped_payments);
        }

        output_payments
    }

    fn distribute_payments(
        &self,
        dest_addresses: ManagedVec<ManagedAddress>,
        payments: ManagedVec<PaymentsVec<Self::Api>>,
    ) {
        for (dest, user_tokens) in dest_addresses.iter().zip(payments.iter()) {
            self.send().direct_multi(&dest, &user_tokens);
        }
    }

    // proxies

    #[proxy]
    fn wrapping_contract_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> bridged_tokens_wrapper::Proxy<Self::Api>;

    fn get_wrapping_contract_proxy_instance(&self) -> bridged_tokens_wrapper::Proxy<Self::Api> {
        self.wrapping_contract_proxy(self.wrapping_contract_address().get())
    }

    // storage

    #[view(getWrappingContractAddress)]
    #[storage_mapper("wrappingContractAddress")]
    fn wrapping_contract_address(&self) -> SingleValueMapper<ManagedAddress>;

    // events

    #[event("transferPerformedEvent")]
    fn transfer_performed_event(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("transferFailedInvalidToken")]
    fn transfer_failed_invalid_token(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("transferFailedFrozenDestinationAccount")]
    fn transfer_failed_frozen_destination_account(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
    );

    #[event("transferOverMaxAmount")]
    fn transfer_over_max_amount(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);
}
