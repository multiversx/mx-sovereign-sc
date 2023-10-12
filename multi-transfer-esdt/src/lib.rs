#![no_std]

multiversx_sc::imports!();

use core::ops::Deref;

use transaction::{
    BatchId, GasLimit, PaymentsVec, StolenFromFrameworkEsdtTokenData, Transaction,
    TxBatchSplitInFields, TxId, TxNonce,
};
use tx_batch_module::FIRST_BATCH_ID;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = u64::MAX;
const NFT_AMOUNT: u32 = 1;

const CALLBACK_GAS: GasLimit = 1_000_000; // Increase if not enough

#[multiversx_sc::contract]
pub trait MultiTransferEsdt:
    tx_batch_module::TxBatchModule + max_bridged_amount_module::MaxBridgedAmountModule
{
    #[init]
    fn init(&self) {
        self.max_tx_batch_size().set(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set(FIRST_BATCH_ID);
        self.last_batch_id().set(FIRST_BATCH_ID);
    }

    #[endpoint]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        batch_id: BatchId,
        transfers: MultiValueEncoded<Transaction<Self::Api>>,
    ) {
        let mut successful_tx_list = ManagedVec::new();
        let mut all_tokens_to_send = ManagedVec::new();
        let mut refund_tx_list = ManagedVec::new();

        let own_sc_address = self.blockchain().get_sc_address();
        let sc_shard = self.blockchain().get_shard_of_address(&own_sc_address);

        for sov_tx in transfers {
            let mut refund_tokens_for_user = ManagedVec::new();
            let mut tokens_to_send = ManagedVec::new();
            let mut sent_token_data = ManagedVec::new();

            for (token, token_data) in sov_tx.tokens.iter().zip(sov_tx.token_data.iter()) {
                let must_refund =
                    self.check_must_refund(&token, &sov_tx.to, batch_id, sov_tx.nonce, sc_shard);

                if must_refund {
                    refund_tokens_for_user.push(token);
                } else {
                    tokens_to_send.push(token);
                    sent_token_data.push(token_data);
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
            all_tokens_to_send.push(user_tokens_to_send);

            successful_tx_list.push(sov_tx);
        }

        self.distribute_payments(batch_id, successful_tx_list, all_tokens_to_send);

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

    // private

    fn check_must_refund(
        &self,
        token: &EsdtTokenPayment,
        dest: &ManagedAddress,
        batch_id: BatchId,
        tx_nonce: TxNonce,
        sc_shard: u32,
    ) -> bool {
        if token.token_nonce == 0 {
            if !self.is_local_role_set(&token.token_identifier, &EsdtLocalRole::Mint) {
                self.transfer_failed_invalid_token(batch_id, tx_nonce);

                return true;
            }
        } else if !self.has_nft_roles(token) {
            self.transfer_failed_invalid_token(batch_id, tx_nonce);

            return true;
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

    fn has_nft_roles(&self, payment: &EsdtTokenPayment) -> bool {
        if !self.is_local_role_set(&payment.token_identifier, &EsdtLocalRole::NftCreate) {
            return false;
        }

        if payment.amount > NFT_AMOUNT
            && !self.is_local_role_set(&payment.token_identifier, &EsdtLocalRole::NftAddQuantity)
        {
            return false;
        }

        true
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
        all_token_data: ManagedVec<StolenFromFrameworkEsdtTokenData<Self::Api>>,
    ) -> PaymentsVec<Self::Api> {
        let mut output_payments = PaymentsVec::new();
        for (payment, token_data) in payments.iter().zip(all_token_data.iter()) {
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

    fn distribute_payments(
        &self,
        batch_id: BatchId,
        tx_list: ManagedVec<Transaction<Self::Api>>,
        tokens_list: ManagedVec<PaymentsVec<Self::Api>>,
    ) {
        for (tx, payments) in tx_list.iter().zip(tokens_list.iter()) {
            match &tx.opt_transfer_data {
                Some(tx_data) => {
                    let mut args = ManagedArgBuffer::new();
                    for arg in &tx_data.args {
                        args.push_arg(arg);
                    }

                    self.send()
                        .contract_call::<()>(tx.to.clone(), tx_data.function.clone())
                        .with_raw_arguments(args)
                        .with_multi_token_transfer(payments.deref().clone())
                        .with_gas_limit(tx_data.gas_limit)
                        .async_call_promise()
                        .with_extra_gas_for_callback(CALLBACK_GAS)
                        .with_callback(self.callbacks().transfer_callback(batch_id, tx.nonce, tx))
                        .register_promise();
                }
                None => {
                    self.send().direct_multi(&tx.to, payments.deref());

                    self.transfer_performed_event(batch_id, tx.nonce, tx);
                }
            }
        }
    }

    #[promises_callback]
    fn transfer_callback(
        &self,
        batch_id: BatchId,
        tx_nonce: TxNonce,
        original_tx: Transaction<Self::Api>,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                self.transfer_performed_event(batch_id, tx_nonce, original_tx);
            }
            ManagedAsyncCallResult::Err(_) => {
                let tokens = original_tx.tokens.clone();
                let refund_tx = self.convert_to_refund_tx(original_tx, tokens);
                self.add_multiple_tx_to_batch(&ManagedVec::from_single_item(refund_tx));

                self.transfer_failed_execution_failed(batch_id, tx_nonce);
            }
        }
    }

    // events

    #[event("transferPerformedEvent")]
    fn transfer_performed_event(
        &self,
        #[indexed] batch_id: BatchId,
        #[indexed] tx_id: TxId,
        tx: Transaction<Self::Api>,
    );

    #[event("transferFailedInvalidToken")]
    fn transfer_failed_invalid_token(&self, #[indexed] batch_id: BatchId, #[indexed] tx_id: TxId);

    #[event("transferFailedFrozenDestinationAccount")]
    fn transfer_failed_frozen_destination_account(
        &self,
        #[indexed] batch_id: BatchId,
        #[indexed] tx_id: TxId,
    );

    #[event("transferOverMaxAmount")]
    fn transfer_over_max_amount(&self, #[indexed] batch_id: BatchId, #[indexed] tx_id: TxId);

    #[event("transferFailedExecutionFailed")]
    fn transfer_failed_execution_failed(
        &self,
        #[indexed] batch_id: BatchId,
        #[indexed] tx_id: TxId,
    );
}
