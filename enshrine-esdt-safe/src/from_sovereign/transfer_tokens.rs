use header_verifier::header_verifier_proxy;
use multiversx_sc::{api::ESDT_MULTI_TRANSFER_FUNC_NAME, storage::StorageKey};
use transaction::{
    BatchId, GasLimit, Operation, OperationData, OperationEsdtPayment, OperationTuple,
};

use crate::{
    custom_builtin_func_proxy,
    to_sovereign::{self, create_tx::ESDT_SYSTEM_SC_ADDRESS},
};

use super::token_mapping::EsdtTokenInfo;

pub const ESDT_NFT_CREATE_FUNC_NAME: &str = "ESDTNFTCreate";

multiversx_sc::imports!();

const CALLBACK_GAS: GasLimit = 10_000_000; // Increase if not enough
const TRANSACTION_GAS: GasLimit = 30_000_000;

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
    + to_sovereign::events::EventsModule
{
    #[endpoint(executeBridgeOps)]
    fn execute_operations(&self, hash_of_hashes: ManagedBuffer, operation: Operation<Self::Api>) {
        require!(
            !self.is_sovereign_chain().get(),
            "Invalid method to call in current chain"
        );

        require!(self.not_paused(), "Cannot transfer while paused");

        let (operation_hash, is_registered) =
            self.calculate_operation_hash(hash_of_hashes.clone(), operation.clone());

        if !is_registered {
            sc_panic!("Operation is not registered");
        }

        let minted_operation_tokens = self.mint_tokens(&operation.tokens);
        let operation_tuple = OperationTuple {
            op_hash: operation_hash,
            operation,
        };

        self.distribute_payments(hash_of_hashes, operation_tuple, minted_operation_tokens);
    }

    fn call_system_sc_nft_create(&self, operation_token: &OperationEsdtPayment<Self::Api>) -> u64 {
        self.tx()
            .to(ManagedAddress::from(ESDT_SYSTEM_SC_ADDRESS))
            .typed(custom_builtin_func_proxy::UserBuiltinProxy)
            .esdt_nft_create(
                &operation_token.token_identifier,
                &operation_token.token_data.amount,
                &operation_token.token_data.name,
                &operation_token.token_data.royalties,
                &operation_token.token_data.hash,
                &operation_token.token_data.attributes,
                &operation_token.token_data.uris,
                &operation_token.token_nonce,
                &operation_token.token_data.creator,
            )
            .returns(ReturnsResult)
            .sync_call()
    }

    fn mint_tokens(
        &self,
        operation_tokens: &ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) -> ManagedVec<OperationEsdtPayment<Self::Api>> {
        let mut output_payments = ManagedVec::new();

        for operation_token in operation_tokens.iter() {
            if !self.has_sov_token_prefix(&operation_token.token_identifier) {
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
                nonce = self.call_system_sc_nft_create(&operation_token);

                // nonce = self.send().esdt_nft_create(
                //     &operation_token.token_identifier,
                //     &operation_token.token_data.amount,
                //     &operation_token.token_data.name,
                //     &operation_token.token_data.royalties,
                //     &operation_token.token_data.hash,
                //     &operation_token.token_data.attributes,
                //     &operation_token.token_data.uris,
                // );

                // let token_data = operation_token.token_data.clone();
                // let mut arg_buffer = ManagedArgBuffer::new();
                // arg_buffer.push_arg(&operation_token.token_identifier);
                // arg_buffer.push_arg(token_data.amount);
                // arg_buffer.push_arg(token_data.name);
                // arg_buffer.push_arg(token_data.royalties);
                // arg_buffer.push_arg(token_data.hash);
                // arg_buffer.push_arg(token_data.attributes);
                // let uris = token_data.uris.clone();

                // if uris.is_empty() {
                //     // at least one URI is required, so we push an empty one
                //     arg_buffer.push_arg(codec::Empty);
                // } else {
                //     // The API function has the last argument as variadic,
                //     // so we top-encode each and send as separate argument
                //     for uri in &uris {
                //         arg_buffer.push_arg(uri);
                //     }
                // }
                // arg_buffer.push_arg(operation_token.token_nonce);

                // self.send_raw().call_local_esdt_built_in_function(
                //     self.blockchain().get_gas_left(),
                //     &ManagedBuffer::from(ESDT_NFT_CREATE_FUNC_NAME),
                //     &arg_buffer,
                // );
            }

            output_payments.push(OperationEsdtPayment {
                token_identifier: operation_token.token_identifier,
                token_nonce: nonce,
                token_data: operation_token.token_data,
            });
        }

        output_payments
    }

    fn mint_and_save_token(
        self,
        mx_token_id: &TokenIdentifier<Self::Api>,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) -> u64 {
        // mint NFT
        let nft_nonce = self.send().esdt_nft_create(
            mx_token_id,
            &operation_token.token_data.amount,
            &operation_token.token_data.name,
            &operation_token.token_data.royalties,
            &operation_token.token_data.hash,
            &operation_token.token_data.attributes,
            &operation_token.token_data.uris,
        );

        // save token id and nonce
        self.sovereign_esdt_token_info_mapper(
            &operation_token.token_identifier,
            &operation_token.token_nonce,
        )
        .set(EsdtTokenInfo {
            token_identifier: mx_token_id.clone(),
            token_nonce: nft_nonce,
        });

        self.multiversx_esdt_token_info_mapper(mx_token_id, &nft_nonce)
            .set(EsdtTokenInfo {
                token_identifier: operation_token.token_identifier.clone(),
                token_nonce: operation_token.token_nonce,
            });

        nft_nonce
    }

    fn distribute_payments(
        &self,
        hash_of_hashes: ManagedBuffer,
        operation_tuple: OperationTuple<Self::Api>,
        tokens_list: ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) {
        let mapped_tokens = tokens_list.iter().map(|token| token.into()).collect();

        match &operation_tuple.operation.data.opt_transfer_data {
            Some(transfer_data) => {
                let mut args = ManagedArgBuffer::new();
                for arg in &transfer_data.args {
                    args.push_arg(arg);
                }

                self.send()
                    .contract_call::<()>(
                        operation_tuple.operation.to.clone(),
                        transfer_data.function.clone(),
                    )
                    .with_raw_arguments(args)
                    .with_multi_token_transfer(mapped_tokens)
                    .with_gas_limit(transfer_data.gas_limit)
                    .async_call_promise()
                    .with_extra_gas_for_callback(CALLBACK_GAS)
                    .with_callback(
                        <Self as TransferTokensModule>::callbacks(self)
                            .execute(&hash_of_hashes, &operation_tuple),
                    )
                    .register_promise();
            }
            None => {
                let own_address = self.blockchain().get_sc_address();

                self.send()
                    .contract_call::<()>(own_address, ESDT_MULTI_TRANSFER_FUNC_NAME)
                    .with_raw_arguments(
                        self.get_contract_call_args(&operation_tuple.operation.to, mapped_tokens),
                    )
                    .with_gas_limit(TRANSACTION_GAS)
                    .async_call_promise()
                    .with_callback(
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
                self.emit_transfer_failed_events(hash_of_hashes, operation_tuple);
            }
        }

        let caller = self.blockchain().get_caller();
        let header_verifier_address = self.header_verifier_address().get();

        let _ = self
            .tx()
            .from(caller)
            .to(header_verifier_address)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .remove_executed_hash(hash_of_hashes, &operation_tuple.op_hash);
    }

    fn emit_transfer_failed_events(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
    ) {
        // confirmation event
        self.execute_bridge_operation_event(
            hash_of_hashes.clone(),
            operation_tuple.op_hash.clone(),
        );

        for operation_token in &operation_tuple.operation.tokens {
            let mx_token_id_state = self
                .sovereign_to_multiversx_token_id(&operation_token.token_identifier)
                .get();

            if let TokenMapperState::Token(mx_token_id) = mx_token_id_state {
                let mut mx_token_nonce = 0;

                if operation_token.token_nonce > 0 {
                    mx_token_nonce = self
                        .sovereign_esdt_token_info_mapper(
                            &operation_token.token_identifier,
                            &operation_token.token_nonce,
                        )
                        .take()
                        .token_nonce;

                    self.multiversx_esdt_token_info_mapper(&mx_token_id, &mx_token_nonce);
                }

                self.send().esdt_local_burn(
                    &mx_token_id,
                    mx_token_nonce,
                    &operation_token.token_data.amount,
                );
            }
        }

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

    #[storage_mapper("nextBatchId")]
    fn next_batch_id(&self) -> SingleValueMapper<BatchId>;

    #[storage_mapper("pending_hashes")]
    fn pending_hashes(&self, hash_of_hashes: &ManagedBuffer) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("header_verifier_address")]
    fn header_verifier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("sovereign_bridge_address")]
    fn sovereign_bridge_address(&self) -> SingleValueMapper<ManagedAddress>;
}
