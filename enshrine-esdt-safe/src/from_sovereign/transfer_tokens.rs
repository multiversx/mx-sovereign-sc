use crate::{common, to_sovereign, token_handler_proxy};
use multiversx_sc::imports::*;
use multiversx_sc::storage::StorageKey;
use transaction::{Operation, OperationData, OperationEsdtPayment, OperationTuple};

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

        let mut managed_tokens = MultiValueEncoded::new();

        for token in operation.tokens.iter() {
            managed_tokens.push(token);
        }

        let token_handler_address = self.token_handler_address().get();

        self.tx()
            .to(token_handler_address)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .mint_tokens(
                hash_of_hashes,
                OperationTuple {
                    op_hash: operation_hash,
                    operation,
                },
                managed_tokens,
            )
            .sync_call();
    }

    #[endpoint]
    fn call_token_handler_mint_endpoint(
        &self,
        hash_of_hashes: ManagedBuffer<Self::Api>,
        operation_tuple: OperationTuple<Self::Api>,
    ) {
        let token_handler_address = self.token_handler_address().get();
        let multi_value_tokens: MultiValueEncoded<Self::Api, OperationEsdtPayment<Self::Api>> =
            operation_tuple.operation.tokens.clone().into();

        self.tx()
            .to(token_handler_address)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .mint_tokens(hash_of_hashes, operation_tuple, multi_value_tokens)
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
