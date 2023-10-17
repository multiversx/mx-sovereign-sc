use core::ops::Deref;

use transaction::{
    BatchId, GasLimit, PaymentsVec, StolenFromFrameworkEsdtTokenData, Transaction, TxNonce,
};

use bls_signature::BlsSignature;

use crate::from_sovereign::refund::CheckMustRefundArgs;

multiversx_sc::imports!();

const CALLBACK_GAS: GasLimit = 1_000_000; // Increase if not enough

#[multiversx_sc::module]
pub trait TransferTokensModule:
    bls_signature::BlsSignatureModule
    + super::events::EventsModule
    + super::refund::RefundModule
    + super::token_mapping::TokenMappingModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::pause::PauseModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        batch_id: BatchId,
        signature: BlsSignature<Self::Api>,
        transfers: MultiValueEncoded<Transaction<Self::Api>>,
    ) {
        require!(self.not_paused(), "Cannot transfer while paused");

        let next_batch_id = self.next_batch_id().get();
        require!(batch_id == next_batch_id, "Unexpected batch ID");

        let mut successful_tx_list = ManagedVec::new();
        let mut all_tokens_to_send = ManagedVec::new();
        let mut refund_tx_list = ManagedVec::new();

        let signed_transactions = self.verify_bls_signature(transfers, &signature);

        let own_sc_address = self.blockchain().get_sc_address();
        let sc_shard = self.blockchain().get_shard_of_address(&own_sc_address);

        for sov_tx in &signed_transactions {
            let mut refund_tokens_for_user = ManagedVec::new();
            let mut tokens_to_send = ManagedVec::new();
            let mut sent_token_data = ManagedVec::new();

            for (token, token_data) in sov_tx.tokens.iter().zip(sov_tx.token_data.iter()) {
                let token_roles = self
                    .blockchain()
                    .get_esdt_local_roles(&token.token_identifier);
                let must_refund_args = CheckMustRefundArgs {
                    token: &token,
                    roles: token_roles,
                    dest: &sov_tx.to,
                    batch_id,
                    tx_nonce: sov_tx.nonce,
                    sc_address: &own_sc_address,
                    sc_shard,
                };
                let must_refund = self.check_must_refund(must_refund_args);

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

        self.next_batch_id().set(batch_id + 1);
    }

    fn mint_tokens(
        &self,
        payments: PaymentsVec<Self::Api>,
        all_token_data: ManagedVec<StolenFromFrameworkEsdtTokenData<Self::Api>>,
    ) -> PaymentsVec<Self::Api> {
        let own_sc_address = self.blockchain().get_sc_address();
        let mut output_payments = PaymentsVec::new();
        for (payment, token_data) in payments.iter().zip(all_token_data.iter()) {
            let token_balance = self.blockchain().get_esdt_balance(
                &own_sc_address,
                &payment.token_identifier,
                payment.token_nonce,
            );
            if token_balance >= payment.amount {
                output_payments.push(payment);

                continue;
            }

            let mx_token_id_state = self
                .sovereign_to_multiversx_token_id(&payment.token_identifier)
                .get();

            let mx_token_id = match mx_token_id_state {
                TokenMapperState::Token(token_id) => token_id,
                _ => sc_panic!("No token config set!"),
            };

            if payment.token_nonce == 0 {
                self.send()
                    .esdt_local_mint(&mx_token_id, 0, &payment.amount);

                output_payments.push(EsdtTokenPayment::new(mx_token_id, 0, payment.amount));

                continue;
            }

            let token_nonce = self.send().esdt_nft_create(
                &mx_token_id,
                &payment.amount,
                &token_data.name,
                &token_data.royalties,
                &token_data.hash,
                &token_data.attributes,
                &token_data.uris,
            );
            output_payments.push(EsdtTokenPayment::new(
                mx_token_id,
                token_nonce,
                payment.amount,
            ));
        }

        output_payments
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
                        .with_callback(
                            <Self as TransferTokensModule>::callbacks(self)
                                .transfer_callback(batch_id, tx.nonce, tx),
                        )
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
                self.add_to_batch(refund_tx);

                self.transfer_failed_execution_failed(batch_id, tx_nonce);
            }
        }
    }

    #[storage_mapper("nextBatchId")]
    fn next_batch_id(&self) -> SingleValueMapper<BatchId>;
}
