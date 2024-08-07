use crate::{common, to_sovereign, token_handler_proxy};
use builtin_func_names::ESDT_NFT_CREATE_FUNC_NAME;
use header_verifier::header_verifier_proxy;
use multiversx_sc::imports::*;
use multiversx_sc::{api::ESDT_MULTI_TRANSFER_FUNC_NAME, codec, storage::StorageKey};
use transaction::{GasLimit, Operation, OperationData, OperationEsdtPayment, OperationTuple};

const CALLBACK_GAS: GasLimit = 10_000_000; // Increase if not enough
const TRANSACTION_GAS: GasLimit = 30_000_000;
const DEFAULT_ISSUE_COST: u64 = 50000000000000000;

#[multiversx_sc::module]
pub trait TransferTokensModule:
    bls_signature::BlsSignatureModule
    + super::events::EventsModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::pause::PauseModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + utils::UtilsModule
    + to_sovereign::events::EventsModule
    + common::storage::CommonStorage
{
    #[endpoint(executeBridgeOps)]
    fn execute_operations(&self, hash_of_hashes: ManagedBuffer, operation: Operation<Self::Api>) {
        let is_sovereign_chain = self.is_sovereign_chain().get();

        require!(
            !is_sovereign_chain,
            "Invalid method to call in current chain"
        );

        require!(self.not_paused(), "Cannot transfer while paused");

        let (operation_hash, is_registered) =
            self.calculate_operation_hash(hash_of_hashes.clone(), operation.clone());

        if !is_registered {
            sc_panic!("Operation is not registered");
        }

        let are_tokens_registered =
            self.verify_operation_tokens_issue_paid(operation.tokens.clone());

        if !are_tokens_registered {
            self.emit_transfer_failed_events(
                &hash_of_hashes,
                &OperationTuple {
                    op_hash: operation_hash.clone(),
                    operation: operation.clone(),
                },
            );

            return;
        }
    }

    #[endpoint]
    fn call_token_handler_mint_endpoint(&self, operation: Operation<Self::Api>) {
        let token_handler_address = self.token_handler_address().get();

        self.tx()
            .to(token_handler_address)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .mint_tokens(MultiValueEncoded::from(operation.tokens.clone()))
            .callback(<Self as TransferTokensModule>::callbacks(self).save_minted_tokens())
            .async_call_and_exit();
    }

    #[promises_callback]
    fn save_minted_tokens(
        &self,
        #[call_result] result: ManagedAsyncCallResult<
            MultiValueEncoded<OperationEsdtPayment<Self::Api>>,
        >,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(tokens) => {
                for token in tokens {
                    self.minted_tokens().push(&token);
                }
            }
            ManagedAsyncCallResult::Err(_) => {
                sc_panic!("Error at minting tokens");
            }
        }
    }

    #[endpoint(registerNewTokenID)]
    #[payable("*")]
    fn register_new_token_id(&self, tokens: MultiValueEncoded<TokenIdentifier>) {
        let call_payment = self.call_value().single_esdt().clone();
        let wegld_identifier = self.wegld_identifier().get();

        require!(
            call_payment.token_identifier == wegld_identifier,
            "WEGLD is the only token accepted as register fee"
        );

        require!(
            call_payment.amount == DEFAULT_ISSUE_COST * tokens.len() as u64,
            "WEGLD fee amount is not met"
        );

        for token_id in tokens {
            self.register_token(token_id);
        }
    }

    fn verify_operation_tokens_issue_paid(
        &self,
        tokens: ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) -> bool {
        let sov_prefix = self.get_sovereign_prefix();

        for token in tokens.iter() {
            if !self.has_sov_prefix(&token.token_identifier, &sov_prefix) {
                continue;
            }

            if !self.paid_issued_tokens().contains(&token.token_identifier) {
                return false;
            }
        }

        true
    }

    fn mint_tokens(
        &self,
        operation_tokens: &ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) -> ManagedVec<OperationEsdtPayment<Self::Api>> {
        let mut output_payments = ManagedVec::new();
        let sov_prefix = self.get_sovereign_prefix();

        for operation_token in operation_tokens.iter() {
            if !self.has_sov_prefix(&operation_token.token_identifier, &sov_prefix) {
                output_payments.push(operation_token.clone());
                continue;
            }

            let mut nonce = operation_token.token_nonce;
            if nonce == 0 {
                self.send().esdt_local_mint(
                    &operation_token.token_identifier,
                    operation_token.token_nonce,
                    &operation_token.token_data.amount,
                );
            } else {
                let token_data = operation_token.token_data.clone();
                let mut arg_buffer = ManagedArgBuffer::new();

                arg_buffer.push_arg(&operation_token.token_identifier);
                arg_buffer.push_arg(token_data.amount);
                arg_buffer.push_arg(token_data.name);
                arg_buffer.push_arg(token_data.royalties);
                arg_buffer.push_arg(token_data.hash);
                arg_buffer.push_arg(token_data.attributes);

                let uris = token_data.uris.clone();

                if uris.is_empty() {
                    // at least one URI is required, so we push an empty one
                    arg_buffer.push_arg(codec::Empty);
                } else {
                    // The API function has the last argument as variadic,
                    // so we top-encode each and send as separate argument
                    for uri in &uris {
                        arg_buffer.push_arg(uri);
                    }
                }

                arg_buffer.push_arg(operation_token.token_nonce);
                arg_buffer.push_arg(token_data.creator);

                let output = self.send_raw().call_local_esdt_built_in_function(
                    self.blockchain().get_gas_left(),
                    &ManagedBuffer::from(ESDT_NFT_CREATE_FUNC_NAME),
                    &arg_buffer,
                );

                if let Some(first_result_bytes) = output.try_get(0) {
                    nonce = first_result_bytes.parse_as_u64().unwrap_or_default()
                } else {
                    nonce = 0
                }
            }

            output_payments.push(OperationEsdtPayment {
                token_identifier: operation_token.token_identifier,
                token_nonce: nonce,
                token_data: operation_token.token_data,
            });
        }

        output_payments
    }
    fn distribute_payments(
        &self,
        hash_of_hashes: ManagedBuffer,
        operation_tuple: OperationTuple<Self::Api>,
        tokens_list: ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) {
        let mapped_tokens: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> =
            tokens_list.iter().map(|token| token.into()).collect();

        match &operation_tuple.operation.data.opt_transfer_data {
            Some(transfer_data) => {
                let mut args = ManagedArgBuffer::new();
                for arg in &transfer_data.args {
                    args.push_arg(arg);
                }

                self.tx()
                    .to(&operation_tuple.operation.to)
                    .raw_call(transfer_data.function.clone())
                    .arguments_raw(args.clone())
                    .multi_esdt(mapped_tokens.clone())
                    .gas(transfer_data.gas_limit)
                    .callback(
                        <Self as TransferTokensModule>::callbacks(self)
                            .execute(&hash_of_hashes, &operation_tuple),
                    )
                    .gas_for_callback(CALLBACK_GAS)
                    .register_promise();
            }
            None => {
                let own_address = self.blockchain().get_sc_address();
                let args =
                    self.get_contract_call_args(&operation_tuple.operation.to, mapped_tokens);

                self.tx()
                    .to(own_address)
                    .raw_call(ESDT_MULTI_TRANSFER_FUNC_NAME)
                    .arguments_raw(args)
                    .gas(TRANSACTION_GAS)
                    .callback(
                        <Self as TransferTokensModule>::callbacks(self)
                            .execute(&hash_of_hashes, &operation_tuple),
                    )
                    .register_promise();
            }
        }
    }

    fn get_contract_call_args(
        self,
        to: &ManagedAddress,
        mapped_tokens: ManagedVec<EsdtTokenPayment<Self::Api>>,
    ) -> ManagedArgBuffer<Self::Api> {
        let mut args = ManagedArgBuffer::new();
        args.push_arg(to);
        args.push_arg(mapped_tokens.len());

        for token in &mapped_tokens {
            args.push_arg(token.token_identifier);
            args.push_arg(token.token_nonce);
            args.push_arg(token.amount);
        }

        args
    }

    #[promises_callback]
    fn execute(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                self.execute_bridge_operation_event(
                    hash_of_hashes.clone(),
                    operation_tuple.op_hash.clone(),
                );
            }
            ManagedAsyncCallResult::Err(_) => {
                self.burn_sovereign_tokens(&operation_tuple.operation);
                self.emit_transfer_failed_events(hash_of_hashes, operation_tuple);
            }
        }

        let header_verifier_address = self.header_verifier_address().get();

        self.tx()
            .to(header_verifier_address)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .remove_executed_hash(hash_of_hashes, &operation_tuple.op_hash)
            .sync_call();
    }

    fn burn_sovereign_tokens(&self, operation: &Operation<Self::Api>) {
        let sov_prefix = self.get_sovereign_prefix();
        for token in operation.tokens.iter() {
            if !self.has_sov_prefix(&token.token_identifier, &sov_prefix) {
                continue;
            }

            self.send().esdt_local_burn(
                &token.token_identifier,
                token.token_nonce,
                &token.token_data.amount,
            );
        }
    }

    fn emit_transfer_failed_events(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
    ) {
        self.execute_bridge_operation_event(
            hash_of_hashes.clone(),
            operation_tuple.op_hash.clone(),
        );

        // deposit back mainchain tokens into user account
        let sc_address = self.blockchain().get_sc_address();
        let tx_nonce = self.get_and_save_next_tx_id();

        self.deposit_event(
            &operation_tuple.operation.data.op_sender,
            &operation_tuple.operation.get_tokens_as_tuple_arr(),
            OperationData {
                op_nonce: tx_nonce,
                op_sender: sc_address.clone(),
                opt_transfer_data: None,
            },
        );
    }

    // use pending_operations as param
    fn calculate_operation_hash(
        &self,
        hash_of_hashes: ManagedBuffer,
        operation: Operation<Self::Api>,
    ) -> (ManagedBuffer, bool) {
        let mut serialized_data = ManagedBuffer::new();
        let mut storage_key = StorageKey::from("pending_hashes");
        storage_key.append_item(&hash_of_hashes);

        let pending_operations_mapper =
            UnorderedSetMapper::new_from_address(self.header_verifier_address().get(), storage_key);

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

    #[inline]
    fn get_sovereign_prefix(&self) -> ManagedBuffer {
        self.sovereign_tokens_prefix().get()
    }

    #[inline]
    fn register_token(&self, token_id: TokenIdentifier<Self::Api>) {
        self.paid_issued_tokens().insert(token_id);
    }

    #[inline]
    fn is_wegld(&self, token_id: &TokenIdentifier<Self::Api>) -> bool {
        token_id.eq(&self.wegld_identifier().get())
    }

    #[storage_mapper("pendingHashes")]
    fn pending_hashes(&self, hash_of_hashes: &ManagedBuffer) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("headerVerifierAddress")]
    fn header_verifier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("paidIssuedTokens")]
    fn paid_issued_tokens(&self) -> UnorderedSetMapper<TokenIdentifier<Self::Api>>;
}
