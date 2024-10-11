use builtin_func_names::ESDT_MULTI_TRANSFER_FUNC_NAME;
use header_verifier::header_verifier_proxy;
use transaction::{GasLimit, Operation, OperationData, OperationEsdtPayment, OperationTuple};

use crate::to_sovereign;

use super::token_mapping::EsdtInfo;

multiversx_sc::imports!();

const CALLBACK_GAS: GasLimit = 10_000_000; // Increase if not enough
const TRANSACTION_GAS: GasLimit = 30_000_000;

#[multiversx_sc::module]
pub trait TransferTokensModule:
    bls_signature::BlsSignatureModule
    + super::events::EventsModule
    + super::token_mapping::TokenMappingModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::pause::PauseModule
    + utils::UtilsModule
    + to_sovereign::events::EventsModule
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
            self.calculate_operation_hash(&hash_of_hashes, &operation);

        if !is_registered {
            sc_panic!("Operation is not registered");
        }

        let minted_operation_tokens = self.mint_tokens(&operation.tokens);
        let operation_tuple = OperationTuple {
            op_hash: operation_hash,
            operation,
        };

        self.distribute_payments(&hash_of_hashes, &operation_tuple, &minted_operation_tokens);
    }

    fn mint_tokens(
        &self,
        operation_tokens: &ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) -> ManagedVec<OperationEsdtPayment<Self::Api>> {
        let mut output_payments = ManagedVec::new();

        for operation_token in operation_tokens.iter() {
            let sov_to_mvx_token_id_mapper =
                self.sovereign_to_multiversx_token_id_mapper(&operation_token.token_identifier);

            // token is from mainchain -> push token
            if sov_to_mvx_token_id_mapper.is_empty() {
                output_payments.push(operation_token.clone());

                continue;
            }

            // token is from sovereign -> continue and mint
            let mvx_token_id = sov_to_mvx_token_id_mapper.get();

            if operation_token.token_nonce == 0 {
                self.tx()
                    .to(ToSelf)
                    .typed(system_proxy::UserBuiltinProxy)
                    .esdt_local_mint(&mvx_token_id, 0, &operation_token.token_data.amount)
                    .transfer_execute();

                output_payments.push(OperationEsdtPayment {
                    token_identifier: mvx_token_id,
                    token_nonce: 0,
                    token_data: operation_token.token_data,
                });

                continue;
            }

            let nft_nonce = self.mint_and_save_token(&mvx_token_id, &operation_token);

            output_payments.push(OperationEsdtPayment {
                token_identifier: mvx_token_id,
                token_nonce: nft_nonce,
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
        let nft_nonce = self
            .tx()
            .to(ToSelf)
            .typed(system_proxy::UserBuiltinProxy)
            .esdt_nft_create(
                mx_token_id,
                &operation_token.token_data.amount,
                &operation_token.token_data.name,
                &operation_token.token_data.royalties,
                &operation_token.token_data.hash,
                &operation_token.token_data.attributes,
                &operation_token.token_data.uris,
            )
            .returns(ReturnsResult)
            .sync_call();

        // save token id and nonce
        self.sovereign_to_multiversx_esdt_info_mapper(
            &operation_token.token_identifier,
            &operation_token.token_nonce,
        )
        .set(EsdtInfo {
            token_identifier: mx_token_id.clone(),
            token_nonce: nft_nonce,
        });

        self.multiversx_to_sovereign_esdt_info_mapper(mx_token_id, &nft_nonce)
            .set(EsdtInfo {
                token_identifier: operation_token.token_identifier.clone(),
                token_nonce: operation_token.token_nonce,
            });

        nft_nonce
    }

    // TODO: create a callback module
    fn distribute_payments(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
        tokens_list: &ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) {
        let mapped_tokens: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> =
            tokens_list.iter().map(|token| token.into()).collect();

        match &operation_tuple.operation.data.opt_transfer_data {
            Some(transfer_data) => {
                let args = ManagedArgBuffer::from(transfer_data.args.clone());

                self.tx()
                    .to(&operation_tuple.operation.to)
                    .raw_call(transfer_data.function.clone())
                    .arguments_raw(args)
                    .multi_esdt(mapped_tokens.clone())
                    .gas(transfer_data.gas_limit)
                    .callback(
                        <Self as TransferTokensModule>::callbacks(self)
                            .execute(hash_of_hashes, operation_tuple),
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
                            .execute(hash_of_hashes, operation_tuple),
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

        args.push_multi_arg(&mapped_tokens);

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
                self.execute_bridge_operation_event(hash_of_hashes, &operation_tuple.op_hash);
            }
            ManagedAsyncCallResult::Err(_) => {
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

    fn emit_transfer_failed_events(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
    ) {
        // confirmation event
        self.execute_bridge_operation_event(hash_of_hashes, &operation_tuple.op_hash);

        for operation_token in &operation_tuple.operation.tokens {
            let sov_to_mvx_token_id_mapper =
                self.sovereign_to_multiversx_token_id_mapper(&operation_token.token_identifier);

            if !sov_to_mvx_token_id_mapper.is_empty() {
                let mvx_token_id = sov_to_mvx_token_id_mapper.get();
                let mut mx_token_nonce = 0;

                if operation_token.token_nonce > 0 {
                    mx_token_nonce = self
                        .sovereign_to_multiversx_esdt_info_mapper(
                            &operation_token.token_identifier,
                            &operation_token.token_nonce,
                        )
                        .take()
                        .token_nonce;

                    self.multiversx_to_sovereign_esdt_info_mapper(&mvx_token_id, &mx_token_nonce)
                        .take();
                }

                self.tx()
                    .to(ToSelf)
                    .typed(system_proxy::UserBuiltinProxy)
                    .esdt_local_burn(
                        &mvx_token_id,
                        mx_token_nonce,
                        &operation_token.token_data.amount,
                    )
                    .transfer_execute();
            }
        }

        // deposit back mainchain tokens into user account
        let sc_address = self.blockchain().get_sc_address();
        let tx_nonce = self.get_and_save_next_tx_id();

        self.deposit_event(
            &operation_tuple.operation.data.op_sender,
            &operation_tuple
                .operation
                .map_tokens_to_multi_value_encoded(),
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

    #[storage_mapper("pendingHashes")]
    fn pending_hashes(&self, hash_of_hashes: &ManagedBuffer) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("headerVerifierAddress")]
    fn header_verifier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper_from_address("pendingHashes")]
    fn external_pending_hashes(
        &self,
        sc_address: ManagedAddress,
        hash_of_hashes: &ManagedBuffer,
    ) -> UnorderedSetMapper<ManagedBuffer, ManagedAddress>;
}
