use core::ops::Deref;

use multiversx_sc::storage::StorageKey;
use transaction::{BatchId, GasLimit, Operation, OperationEsdtPayment};

use super::token_mapping::EsdtTokenInfo;

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
    #[endpoint(executeBridgeOps)]
    fn execute_operations(
        &self,
        hash_of_hashes: ManagedBuffer,
        operations: MultiValueEncoded<Operation<Self::Api>>,
    ) {
        require!(self.not_paused(), "Cannot transfer while paused");

        let mut verified_operations = MultiValueEncoded::new();
        let mut minted_operations = ManagedVec::new();

        for operation in &operations.to_vec() {
            let (operation_hash, is_registered) = self.calculate_operation_hash(operation.clone());
            let operation_tuple = MultiValue2::from((operation_hash.clone(), operation.clone()));

            if !is_registered {
                self.emit_transfer_failed_events(
                    &hash_of_hashes,
                    operation_tuple.clone()
                );
            }

            let minted_operation_tokens = self.mint_tokens(&operation.tokens);

            minted_operations.push(minted_operation_tokens);
            verified_operations.push(operation_tuple);

            self.pending_hashes().swap_remove(&operation_hash);
        }

        self.distribute_payments(hash_of_hashes, verified_operations, minted_operations);
    }

    fn mint_tokens(
        &self,
        operation_tokens: &ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) -> ManagedVec<OperationEsdtPayment<Self::Api>> {
        let mut output_payments = ManagedVec::new();

        for operation_token in operation_tokens.iter() {
            let mx_token_id_state = self
                .sovereign_to_multiversx_token_id(&operation_token.token_identifier)
                .get();

            let mx_token_id = match mx_token_id_state {
                TokenMapperState::Token(token_id) => token_id,
                _ => {
                    // TODO: will use sovereign prefix
                    output_payments.push(operation_token);

                    continue;
                }
            };

            if operation_token.token_nonce == 0 {
                self.send()
                    .esdt_local_mint(&mx_token_id, 0, &operation_token.token_data.amount);

                output_payments.push(OperationEsdtPayment {
                    token_identifier: mx_token_id,
                    token_nonce: 0,
                    token_data: operation_token.token_data,
                });

                continue;
            }

            // save this for main -> sov
            let nft_nonce = self.send().esdt_nft_create(
                &mx_token_id,
                &operation_token.token_data.amount,
                &operation_token.token_data.name,
                &operation_token.token_data.royalties,
                &operation_token.token_data.hash,
                &operation_token.token_data.attributes,
                &operation_token.token_data.uris,
            );

            // should register token here
            self.esdt_token_info_mapper(
                &operation_token.token_identifier,
                &operation_token.token_nonce,
            )
            .set(EsdtTokenInfo {
                identifier: mx_token_id.clone(),
                nonce: nft_nonce,
            });

            output_payments.push(OperationEsdtPayment {
                token_identifier: mx_token_id,
                token_nonce: nft_nonce,
                token_data: operation_token.token_data,
            });
        }

        output_payments
    }

    fn distribute_payments(
        &self,
        hash_of_hashes: ManagedBuffer,
        verified_operations: MultiValueEncoded<MultiValue2<ManagedBuffer, Operation<Self::Api>>>,
        tokens_list: ManagedVec<ManagedVec<OperationEsdtPayment<Self::Api>>>,
    ) {
        for (token, operation_tuple) in tokens_list.iter().zip(verified_operations) {
            let mapped_payments = self.map_payments(token.deref().clone());
            let (operation_hash, operation) = operation_tuple.clone().into_tuple();

            match &operation.data.opt_transfer_data {
                Some(transfer_data) => {
                    let mut args = ManagedArgBuffer::new();
                    for arg in &transfer_data.args {
                        args.push_arg(arg);
                    }

                    self.send()
                        .contract_call::<()>(operation.to.clone(), transfer_data.function.clone())
                        .with_raw_arguments(args)
                        .with_multi_token_transfer(mapped_payments)
                        .with_gas_limit(transfer_data.gas_limit)
                        .async_call_promise()
                        .with_extra_gas_for_callback(CALLBACK_GAS)
                        .with_callback(
                            <Self as TransferTokensModule>::callbacks(self)
                                .transfer_callback(hash_of_hashes.clone(), operation_tuple),
                        )
                        .register_promise();
                }
                None => {
                    // does it end execution on fail?
                    self.send().direct_multi(&operation.to, &mapped_payments);

                    self.execute_bridge_operation_event(hash_of_hashes.clone(), operation_hash);
                }
            }
        }
    }

    #[promises_callback]
    fn transfer_callback(
        &self,
        hash_of_hashes: ManagedBuffer,
        operation_tuple: MultiValue2<ManagedBuffer, Operation<Self::Api>>,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        let (operation_hash, _) = operation_tuple.clone().into_tuple();

        match result {
            ManagedAsyncCallResult::Ok(_) => {
                self.execute_bridge_operation_event(hash_of_hashes, operation_hash);
            }
            ManagedAsyncCallResult::Err(_) => {
                self.emit_transfer_failed_events(&hash_of_hashes, operation_tuple);
            }
        }
    }

    fn emit_transfer_failed_events(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: MultiValue2<ManagedBuffer, Operation<Self::Api>>,
    ) {
        let (operation_hash, operation) = operation_tuple.into_tuple();

        self.execute_bridge_operation_event(hash_of_hashes.clone(), operation_hash);

        for operation_token in &operation.tokens {
            let mx_token_id_state = self
                .sovereign_to_multiversx_token_id(&operation_token.token_identifier)
                .get();

            if let TokenMapperState::Token(token_id) = mx_token_id_state {
                if operation_token.token_nonce == 0 {
                    self.send()
                        .esdt_local_burn(&token_id, 0, &operation_token.token_data.amount);

                    continue;
                }

                let esdt_token_info = self
                    .esdt_token_info_mapper(
                        &operation_token.token_identifier,
                        &operation_token.token_nonce,
                    )
                    .get();

                self.send().esdt_local_burn(
                    &esdt_token_info.identifier,
                    esdt_token_info.nonce,
                    &operation_token.token_data.amount,
                );

                self.esdt_token_info_mapper(
                    &operation_token.token_identifier,
                    &operation_token.token_nonce,
                )
                .take();
            }
        }

        // TODO: deposit back to the sender
        // self.deposit_event(
        //     &operation.to,
        //     &tokens_topic,
        //     operation.data
        // );
    }

    // use pending_operations as param
    fn calculate_operation_hash(&self, operation: Operation<Self::Api>) -> (ManagedBuffer, bool) {
        let mut serialized_data = ManagedBuffer::new();

        let pending_operations_mapper = UnorderedSetMapper::new_from_address(
            self.multisig_address().get(),
            StorageKey::from("pending_hashes"),
        );

        if let core::result::Result::Err(err) = operation.top_encode(&mut serialized_data) {
            sc_panic!("Transfer data encode error: {}", err.message_bytes());
        }

        let sha256 = self.crypto().sha256(&serialized_data);

        let hash = sha256.as_managed_buffer().clone();

        if pending_operations_mapper.contains(&hash) {
            (hash, true)
        } else {
            (hash, false)
        }
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
}
