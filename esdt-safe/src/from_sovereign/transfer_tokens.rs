use core::ops::Deref;

use multiversx_sc::storage::StorageKey;
use transaction::{BatchId, GasLimit, Operation, OperationEsdtPayment};

use crate::to_sovereign::events::DepositEvent;

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
    + utils::UtilsModule
{
    #[endpoint(executeOperations)]
    fn execute_operations(&self, operations: MultiValueEncoded<Operation<Self::Api>>) {
        require!(self.not_paused(), "Cannot transfer while paused");

        let mut verified_operations = ManagedVec::new();
        let mut minted_operations = ManagedVec::new();

        for operation in operations {
            let is_hash_ok = self.check_operation_hash(operation.clone());

            if is_hash_ok {
                let minted_operation_tokens = self.mint_tokens(&operation.tokens);

                minted_operations.push(minted_operation_tokens);
                verified_operations.push(operation.clone());

                self.pending_hashes()
                    .swap_remove(&self.calculate_operation_hash(operation));
            }
        }

        self.distribute_payments(verified_operations, minted_operations);
    }

    fn mint_tokens(
        &self,
        operation_tokens: &ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) -> ManagedVec<OperationEsdtPayment<Self::Api>> {
        let own_sc_address = self.blockchain().get_sc_address();
        let mut output_payments = ManagedVec::new();

        for payment in operation_tokens.iter() {
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
                _ => sc_panic!("No token config set!"), // add event
            };

            if payment.token_nonce == 0 {
                self.send()
                    .esdt_local_mint(&mx_token_id, 0, &payment.token_data.amount);

                output_payments.push(OperationEsdtPayment {
                    token_identifier: mx_token_id,
                    token_nonce: 0,
                    token_data: payment.token_data,
                });

                continue;
            }

            let token_nonce = self.send().esdt_nft_create(
                &mx_token_id,
                &payment.token_data.amount,
                &payment.token_data.name,
                &payment.token_data.royalties,
                &payment.token_data.hash,
                &payment.token_data.attributes,
                &payment.token_data.uris,
            );

            output_payments.push(OperationEsdtPayment {
                token_identifier: mx_token_id,
                token_nonce,
                token_data: payment.token_data,
            });
        }

        output_payments
    }

    fn distribute_payments(
        &self,
        verified_operations: ManagedVec<Operation<Self::Api>>,
        tokens_list: ManagedVec<ManagedVec<OperationEsdtPayment<Self::Api>>>,
    ) {
        for (operation, token) in verified_operations.iter().zip(tokens_list.iter()) {
            let mapped_payments = self.map_payments(token.deref().clone());

            match &operation.opt_transfer_data {
                Some(tx_data) => {
                    let mut args = ManagedArgBuffer::new();
                    for arg in &tx_data.args {
                        args.push_arg(arg);
                    }

                    self.send()
                        .contract_call::<()>(operation.to.clone(), tx_data.function.clone())
                        .with_raw_arguments(args)
                        .with_multi_token_transfer(mapped_payments)
                        .with_gas_limit(tx_data.gas_limit)
                        .async_call_promise()
                        .with_extra_gas_for_callback(CALLBACK_GAS)
                        .with_callback(
                            <Self as TransferTokensModule>::callbacks(self)
                                .transfer_callback(operation),
                        )
                        .register_promise();
                }
                None => {
                    self.send().direct_multi(&operation.to, &mapped_payments);

                    self.transfer_performed_event(
                        self.get_hash_of_hashes(),
                        self.calculate_operation_hash(operation),
                    );
                }
            }
        }
    }

    #[promises_callback]
    fn transfer_callback(
        &self,
        operation: Operation<Self::Api>,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                self.transfer_performed_event(
                    self.get_hash_of_hashes(),
                    self.calculate_operation_hash(operation),
                );
            }
            ManagedAsyncCallResult::Err(_) => {
                let tx_nonce = self.get_and_save_next_tx_id();
                let mut tokens_topic = MultiValueEncoded::new();

                for token_payment in operation.tokens.iter() {
                    tokens_topic.push(MultiValue3::from((
                        token_payment.token_identifier,
                        token_payment.token_nonce,
                        token_payment.token_data.amount,
                    )));
                }

                match operation.opt_transfer_data {
                    Some(opt_transfer_data) => self.transfer_failed_execution_failed(
                        &operation.to,
                        &tokens_topic,
                        DepositEvent {
                            tx_nonce,
                            opt_gas_limit: Some(opt_transfer_data.gas_limit),
                            opt_function: Some(opt_transfer_data.function),
                            opt_arguments: Some(opt_transfer_data.args),
                        },
                    ),
                    None => self.transfer_failed_execution_failed(
                        &operation.to,
                        &tokens_topic,
                        DepositEvent {
                            tx_nonce,
                            opt_gas_limit: None,
                            opt_function: None,
                            opt_arguments: None,
                        },
                    ),
                };
            }
        }
    }

    fn check_operation_hash(&self, operation: Operation<Self::Api>) -> bool {
        let multisig_address = self.multisig_address().get();
        let pending_operations_mapper: UnorderedSetMapper<ManagedBuffer, _> =
            UnorderedSetMapper::new_from_address(
                multisig_address,
                StorageKey::from("pending_hashes"),
            );

        let operation_hash = self.calculate_operation_hash(operation);

        pending_operations_mapper.contains(&operation_hash)
    }

    fn calculate_and_set_hash_of_hashes(&self, operations: ManagedVec<Operation<Self::Api>>) {
        let mut all_hashes = ManagedBuffer::new();

        for operation in &operations {
            all_hashes.append(&self.calculate_operation_hash(operation))
        }

        let all_hashes_sha256 = self.crypto().sha256(all_hashes);

        self.hash_of_hashes_mapper()
            .set(all_hashes_sha256.as_managed_buffer().clone());
    }

    fn get_hash_of_hashes(&self) -> ManagedBuffer {
        self.hash_of_hashes_mapper().get()
    }

    fn calculate_operation_hash(&self, operation: Operation<Self::Api>) -> ManagedBuffer {
        let mut serialized_data = ManagedBuffer::new();

        if let core::result::Result::Err(err) = operation.top_encode(&mut serialized_data) {
            sc_panic!("Transfer data encode error: {}", err.message_bytes());
        }

        let sha256 = self.crypto().sha256(&serialized_data);

        sha256.as_managed_buffer().clone()
    }

    fn map_payments(
        &self,
        payments: ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) -> ManagedVec<EsdtTokenPayment<Self::Api>> {
        let mut mapped_payments = ManagedVec::new();

        for payment in &payments {
            let mapped_payment = payment.into();
            mapped_payments.push(mapped_payment);
        }

        mapped_payments
    }

    #[storage_mapper("nextBatchId")]
    fn next_batch_id(&self) -> SingleValueMapper<BatchId>;

    #[storage_mapper("pending_hashes")]
    fn pending_hashes(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("multisig_address")]
    fn multisig_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("hashOfHashes")]
    fn hash_of_hashes_mapper(&self) -> SingleValueMapper<ManagedBuffer>;
}
