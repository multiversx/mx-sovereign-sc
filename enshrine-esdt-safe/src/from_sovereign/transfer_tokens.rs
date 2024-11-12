use crate::{common, to_sovereign, token_handler_proxy};
use multiversx_sc::imports::*;
use proxies::header_verifier_proxy::HeaderverifierProxy;
use transaction::{Operation, OperationData, OperationEsdtPayment, OperationTuple};

const DEFAULT_ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 * 10^18

struct SplitResult<M: ManagedTypeApi> {
    sov_tokens: ManagedVec<M, EsdtTokenPayment<M>>,
    non_sov_tokens: ManagedVec<M, OperationEsdtPayment<M>>,
    are_tokens_registered: bool,
}

impl<M: ManagedTypeApi> Default for SplitResult<M> {
    #[inline]
    fn default() -> Self {
        Self {
            sov_tokens: ManagedVec::new(),
            non_sov_tokens: ManagedVec::new(),
            are_tokens_registered: false,
        }
    }
}

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

        let (op_hash, is_registered) = self.calculate_operation_hash(&hash_of_hashes, &operation);
        if !is_registered {
            sc_panic!("Operation is not registered");
        }

        let split_result = self.split_payments_for_prefix_and_fee(&operation.tokens);
        if !split_result.are_tokens_registered {
            self.emit_transfer_failed_events(
                &hash_of_hashes,
                &OperationTuple::new(op_hash, operation),
            );

            return;
        }

        let token_handler_address = self.token_handler_address().get();
        let multi_value_tokens: MultiValueEncoded<OperationEsdtPayment<Self::Api>> =
            split_result.non_sov_tokens.into();

        self.tx()
            .to(token_handler_address)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .transfer_tokens(
                operation.data.opt_transfer_data,
                operation.to,
                // operation.data.opt_sender
                multi_value_tokens,
            )
            .multi_esdt(split_result.sov_tokens)
            .sync_call();

        self.remove_executed_hash(&hash_of_hashes, &op_hash);
        self.execute_bridge_operation_event(hash_of_hashes, op_hash);
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

    fn split_payments_for_prefix_and_fee(
        &self,
        tokens: &ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) -> SplitResult<Self::Api> {
        let sov_prefix = self.get_sovereign_prefix();
        let mut sov_tokens: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = ManagedVec::new();
        let mut non_sov_tokens: ManagedVec<Self::Api, OperationEsdtPayment<Self::Api>> =
            ManagedVec::new();

        for token in tokens.iter() {
            if !self.has_sov_prefix(&token.token_identifier, &sov_prefix) {
                non_sov_tokens.push(token);

                continue;
            }

            if !self.paid_issued_tokens().contains(&token.token_identifier) {
                return SplitResult::default();
            }

            sov_tokens.push(token.into());
        }

        SplitResult {
            sov_tokens,
            non_sov_tokens,
            are_tokens_registered: true,
        }
    }

    fn remove_executed_hash(&self, hash_of_hashes: &ManagedBuffer, op_hash: &ManagedBuffer) {
        let header_verifier_address = self.header_verifier_address().get();
        self.tx()
            .to(header_verifier_address)
            .typed(HeaderverifierProxy)
            .remove_executed_hash(hash_of_hashes, op_hash)
            .sync_call();
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
            &operation_tuple
                .operation
                .map_tokens_to_multi_value_encoded(),
            OperationData::new(tx_nonce, sc_address.clone(), None),
        );
    }

    // use pending_operations as param
    fn calculate_operation_hash(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation: &Operation<Self::Api>,
    ) -> (ManagedBuffer, bool) {
        let mut serialized_data = ManagedBuffer::new();
        let header_verifier_address = self.header_verifier_address().get();
        let pending_operations_mapper =
            self.external_pending_hashes(header_verifier_address, hash_of_hashes);

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

    #[storage_mapper_from_address("pendingHashes")]
    fn external_pending_hashes(
        &self,
        sc_address: ManagedAddress,
        hash_of_hashes: &ManagedBuffer,
    ) -> UnorderedSetMapper<ManagedBuffer, ManagedAddress>;
}
