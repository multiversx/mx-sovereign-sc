use core::ops::Deref;

use multiversx_sc::storage::StorageKey;
use transaction::{
    BatchId, GasLimit, Operation, OperationEsdtPayment, StolenFromFrameworkEsdtTokenData,
};

use crate::from_sovereign::refund::CheckMustRefundArgs;

multiversx_sc::imports!();

const CALLBACK_GAS: GasLimit = 10_000_000; // Increase if not enough

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
    fn check_operations_hashes(
        self,
        operations: MultiValueEncoded<Operation<Self::Api>>,
    ) -> MultiValueEncoded<Operation<Self::Api>> {
        let mut serialized_transferred_data = ManagedBuffer::new();
        let mut verified_operations: MultiValueEncoded<Self::Api, Operation<Self::Api>> =
            MultiValueEncoded::new();

        let multisig_address = self.multisig_address().get();
        let pending_operations_mapper: UnorderedSetMapper<ManagedBuffer, _> =
            UnorderedSetMapper::new_from_address(
                multisig_address,
                StorageKey::from("pending_hashes"),
            );

        for operation in operations {
            if let core::result::Result::Err(err) =
                operation.top_encode(&mut serialized_transferred_data)
            {
                sc_panic!("Transfer data encode error: {}", err.message_bytes());
            }

            let operation_sha256 = self.crypto().sha256(&serialized_transferred_data);
            let operation_hash = operation_sha256.as_managed_buffer();

            if pending_operations_mapper.contains(operation_hash) {
                verified_operations.push(operation);
            }
        }

        verified_operations
    }

    #[endpoint(executeOperations)]
    fn execute_operations(
        &self,
        batch_id: BatchId,
        operations: MultiValueEncoded<Operation<Self::Api>>,
    ) {
        require!(self.not_paused(), "Cannot transfer while paused");

        let next_batch_id = self.next_batch_id().get();
        require!(batch_id == next_batch_id, "Unexpected batch ID");

        let mut successful_tx_list = ManagedVec::new();
        let mut all_tokens_to_send = ManagedVec::new();

        let own_sc_address = self.blockchain().get_sc_address();
        let sc_shard = self.blockchain().get_shard_of_address(&own_sc_address);

        let verified_operations = self.check_operations_hashes(operations);

        for sov_tx in verified_operations {
            let mut tokens_to_send: ManagedVec<OperationEsdtPayment<Self::Api>> = ManagedVec::new();
            let mut sent_token_data = ManagedVec::new();

            for token in sov_tx.tokens.iter() {
                let token_roles = self
                    .blockchain()
                    .get_esdt_local_roles(&token.token_identifier);
                let must_refund_args = CheckMustRefundArgs {
                    token: &token,
                    roles: token_roles,
                    dest: &sov_tx.to,
                    batch_id,
                    tx_nonce: token.token_nonce,
                    sc_address: &own_sc_address,
                    sc_shard,
                };
                let must_refund = self.check_must_refund(must_refund_args);

                if !must_refund {
                    tokens_to_send.push(token.clone());
                    sent_token_data.push(token.token_data);
                }
            }

            if tokens_to_send.is_empty() {
                continue;
            }

            let user_tokens_to_send = self.mint_tokens(tokens_to_send, sent_token_data);
            all_tokens_to_send.push(user_tokens_to_send);

            successful_tx_list.push(sov_tx);
        }

        self.distribute_payments(batch_id, successful_tx_list, all_tokens_to_send);

        self.next_batch_id().set(batch_id + 1);
    }

    fn mint_tokens(
        &self,
        payments: ManagedVec<OperationEsdtPayment<Self::Api>>,
        all_token_data: ManagedVec<StolenFromFrameworkEsdtTokenData<Self::Api>>,
    ) -> ManagedVec<OperationEsdtPayment<Self::Api>> {
        let own_sc_address = self.blockchain().get_sc_address();
        let mut output_payments = ManagedVec::new();
        for (payment, token_data) in payments.iter().zip(all_token_data.iter()) {
            let token_balance = self.blockchain().get_esdt_balance(
                &own_sc_address,
                &payment.token_identifier,
                payment.token_nonce,
            );
            if token_balance >= payment.token_data.amount {
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
                    .esdt_local_mint(&mx_token_id, 0, &payment.token_data.amount);

                output_payments.push(OperationEsdtPayment {
                    token_identifier: mx_token_id,
                    token_nonce: 0,
                    token_data,
                });
                continue;
            }

            let token_nonce = self.send().esdt_nft_create(
                &mx_token_id,
                &payment.token_data.amount,
                &token_data.name,
                &token_data.royalties,
                &token_data.hash,
                &token_data.attributes,
                &token_data.uris,
            );

            // nonce from nft create or payment?
            output_payments.push(OperationEsdtPayment {
                token_identifier: mx_token_id,
                token_nonce,
                token_data,
            });
        }

        output_payments
    }

    fn map_payments(
        &self, 
        payments: ManagedVec<OperationEsdtPayment<Self::Api>>
    ) -> ManagedVec<EsdtTokenPayment<Self::Api>> {
        let mut mapped_payments = ManagedVec::new();
        for payment in &payments {
            let mapped_payment = payment.into();
            mapped_payments.push(mapped_payment); 
        }

        mapped_payments
    }

    fn distribute_payments(
        &self,
        batch_id: BatchId,
        tx_list: ManagedVec<Operation<Self::Api>>,
        tokens_list: ManagedVec<ManagedVec<OperationEsdtPayment<Self::Api>>>,
    ) {
        for (tx, payments) in tx_list.iter().zip(tokens_list.iter()) {
            let mapped_payments = self.map_payments(payments.deref().clone());
            match &tx.opt_transfer_data {
                Some(tx_data) => {
                    let mut args = ManagedArgBuffer::new();
                    for arg in &tx_data.args {
                        args.push_arg(arg);
                    }

                    self.send()
                        .contract_call::<()>(tx.to.clone(), tx_data.function.clone())
                        .with_raw_arguments(args)
                        .with_multi_token_transfer(mapped_payments)
                        .with_gas_limit(tx_data.gas_limit)
                        .async_call_promise()
                        .with_extra_gas_for_callback(CALLBACK_GAS)
                        .with_callback(
                            <Self as TransferTokensModule>::callbacks(self)
                                .transfer_callback(batch_id, tx),
                        )
                        .register_promise();
                }
                None => {
                    self.send().direct_multi(&tx.to, &mapped_payments);

                    self.transfer_performed_event(batch_id, tx);
                }
            }
        }
    }

    #[promises_callback]
    fn transfer_callback(
        &self,
        batch_id: BatchId,
        // tx_nonce: TxNonce,
        original_tx: Operation<Self::Api>,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                self.transfer_performed_event(batch_id, original_tx);
            }
            ManagedAsyncCallResult::Err(_) => {
                // TODO: rewrite convert_to_refund_tx
                let _tokens = original_tx.tokens.clone();
                // let refund_tx = self.convert_to_refund_tx(original_tx, tokens);
                // self.add_to_batch(refund_tx);

                self.transfer_failed_execution_failed(batch_id);
            }
        }
    }

    #[storage_mapper("nextBatchId")]
    fn next_batch_id(&self) -> SingleValueMapper<BatchId>;

    #[storage_mapper("pending_hashes")]
    fn pending_hashes(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("multisig_address")]
    fn multisig_address(&self) -> SingleValueMapper<ManagedAddress>;
}
