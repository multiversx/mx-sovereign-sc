use builtin_func_names::ESDT_MULTI_TRANSFER_FUNC_NAME;
use header_verifier::header_verifier_proxy;
use transaction::{GasLimit, Operation, OperationData, OperationEsdtPayment, OperationTuple};

use crate::to_sovereign;

multiversx_sc::imports!();

const CALLBACK_GAS: GasLimit = 10_000_000; // Increase if not enough
const ESDT_TRANSACTION_GAS: GasLimit = 5_000_000;

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

        let operation_hash = self.calculate_operation_hash(&operation);

        self.lock_operation_hash(&operation_hash, &hash_of_hashes);

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
            let current_token_type_ref = &operation_token.token_data.token_type;

            if self.is_fungible(current_token_type_ref) {
                self.tx()
                    .to(ToSelf)
                    .typed(system_proxy::UserBuiltinProxy)
                    .esdt_local_mint(&mvx_token_id, 0, &operation_token.token_data.amount)
                    .sync_call();

                output_payments.push(OperationEsdtPayment {
                    token_identifier: mvx_token_id,
                    token_nonce: 0,
                    token_data: operation_token.token_data,
                });

                continue;
            }

            let nft_nonce = self.esdt_create_and_update_mapper(&mvx_token_id, &operation_token);

            output_payments.push(OperationEsdtPayment {
                token_identifier: mvx_token_id,
                token_nonce: nft_nonce,
                token_data: operation_token.token_data,
            });
        }

        output_payments
    }

    fn esdt_create_and_update_mapper(
        self,
        mvx_token_id: &TokenIdentifier<Self::Api>,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) -> u64 {
        let mut nonce = 0;

        let current_token_type_ref = &operation_token.token_data.token_type;

        // if doesn't exist in mapper nonce will be 0 and we need to create the SFT/MetaESDT, otherwise mint
        if self.is_sft_or_meta(current_token_type_ref) {
            nonce = self.get_mvx_nonce_from_mapper(
                &operation_token.token_identifier,
                operation_token.token_nonce,
            )
        }

        // mint NFT
        if nonce == 0 {
            // if NFT/DyNFT => esdt_nft_create
            nonce = self.mint_nft_tx(mvx_token_id, &operation_token.token_data);

            // save token id and nonce
            self.update_esdt_info_mappers(
                &operation_token.token_identifier,
                operation_token.token_nonce,
                mvx_token_id,
                nonce,
            );
        } else {
            // if SFT/DySFT/Meta/DyMeta => esdt_local_mint (add quantity)
            self.tx()
                .to(ToSelf)
                .typed(system_proxy::UserBuiltinProxy)
                .esdt_local_mint(mvx_token_id, nonce, &operation_token.token_data.amount)
                .sync_call();
        }

        nonce
    }

    fn mint_nft_tx(
        &self,
        mvx_token_id: &TokenIdentifier,
        token_data: &EsdtTokenData<Self::Api>,
    ) -> u64 {
        let mut amount = token_data.amount.clone();
        if self.is_sft_or_meta(&token_data.token_type) {
            amount += BigUint::from(1u32);
        }

        self.tx()
            .to(ToSelf)
            .typed(system_proxy::UserBuiltinProxy)
            .esdt_nft_create(
                mvx_token_id,
                &amount,
                &token_data.name,
                &token_data.royalties,
                &token_data.hash,
                &token_data.attributes,
                &token_data.uris,
            )
            .returns(ReturnsResult)
            .sync_call()
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
                    .payment(&mapped_tokens)
                    .gas(transfer_data.gas_limit)
                    .callback(
                        <Self as TransferTokensModule>::callbacks(self)
                            .execute(hash_of_hashes, operation_tuple),
                    )
                    .gas_for_callback(CALLBACK_GAS)
                    .register_promise();
            }
            None => {
                self.tx()
                    .to(&operation_tuple.operation.to)
                    .raw_call(ESDT_MULTI_TRANSFER_FUNC_NAME)
                    .payment(&mapped_tokens)
                    .gas(ESDT_TRANSACTION_GAS)
                    .callback(
                        <Self as TransferTokensModule>::callbacks(self)
                            .execute(hash_of_hashes, operation_tuple),
                    )
                    .gas_for_callback(CALLBACK_GAS)
                    .register_promise();
            }
        }
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
                let mut mvx_token_nonce = 0;

                if operation_token.token_nonce > 0 {
                    mvx_token_nonce = self
                        .sovereign_to_multiversx_esdt_info_mapper(
                            &operation_token.token_identifier,
                            operation_token.token_nonce,
                        )
                        .get()
                        .token_nonce;

                    if self.is_nft(&operation_token.token_data.token_type) {
                        self.clear_sov_to_mvx_esdt_info_mapper(
                            &operation_token.token_identifier,
                            operation_token.token_nonce,
                        );

                        self.clear_mvx_to_sov_esdt_info_mapper(&mvx_token_id, mvx_token_nonce);
                    }
                }

                self.tx()
                    .to(ToSelf)
                    .typed(system_proxy::UserBuiltinProxy)
                    .esdt_local_burn(
                        &mvx_token_id,
                        mvx_token_nonce,
                        &operation_token.token_data.amount,
                    )
                    .sync_call();
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
            OperationData::new(tx_nonce, sc_address.clone(), None),
        );
    }

    fn calculate_operation_hash(&self, operation: &Operation<Self::Api>) -> ManagedBuffer {
        let mut serialized_data = ManagedBuffer::new();

        if let core::result::Result::Err(err) = operation.top_encode(&mut serialized_data) {
            sc_panic!("Transfer data encode error: {}", err.message_bytes());
        }

        let sha256 = self.crypto().sha256(&serialized_data);
        let hash = sha256.as_managed_buffer().clone();

        hash
    }

    fn lock_operation_hash(&self, operation_hash: &ManagedBuffer, hash_of_hashes: &ManagedBuffer) {
        let header_verifier_address = self.header_verifier_address().get();

        self.tx()
            .to(header_verifier_address)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .lock_operation_hash(hash_of_hashes, operation_hash)
            .sync_call();
    }

    fn get_mvx_nonce_from_mapper(self, token_id: &TokenIdentifier, nonce: u64) -> u64 {
        let esdt_info_mapper = self.sovereign_to_multiversx_esdt_info_mapper(token_id, nonce);
        if esdt_info_mapper.is_empty() {
            return 0;
        }
        esdt_info_mapper.get().token_nonce
    }

    #[storage_mapper("headerVerifierAddress")]
    fn header_verifier_address(&self) -> SingleValueMapper<ManagedAddress>;
}
