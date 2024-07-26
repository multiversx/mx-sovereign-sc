use multiversx_sc::api::{ESDT_MULTI_TRANSFER_FUNC_NAME, ESDT_NFT_CREATE_FUNC_NAME};
use multiversx_sc::imports::IgnoreValue;
use multiversx_sc::types::{
    system_proxy, EsdtTokenPayment, ManagedArgBuffer, ManagedAsyncCallResult, ToSelf,
};
use multiversx_sc::types::{ManagedVec, TokenIdentifier};
use multiversx_sc::{codec, err_msg};
use transaction::{
    GasLimit, OperationData, OperationEsdtPayment, OperationTuple, StolenFromFrameworkEsdtTokenData,
};

const CALLBACK_GAS: GasLimit = 10_000_000; // Increase if not enough
const TRANSACTION_GAS: GasLimit = 30_000_000;

use crate::{burn_tokens, common};

#[multiversx_sc::module]
pub trait TransferTokensModule:
    utils::UtilsModule
    + common::storage::CommonStorage
    + common::events::EventsModule
    + burn_tokens::BurnTokensModule
    + tx_batch_module::TxBatchModule
{
    #[endpoint(transferTokens)]
    fn transfer_tokens(
        &self,
        hash_of_hashes: ManagedBuffer,
        operation_tuple: OperationTuple<Self::Api>,
    ) {
        let mut output_payments: ManagedVec<Self::Api, OperationEsdtPayment<Self::Api>> =
            ManagedVec::new();

        for operation_token in operation_tuple.operation.tokens.iter() {
            let sov_prefix = self.sov_prefix().get();

            if !self.has_sov_prefix(&operation_token.token_identifier, &sov_prefix) {
                output_payments.push(operation_token.clone());
                continue;
            }

            let mut nonce = operation_token.token_nonce;

            if nonce == 0 {
                self.tx()
                    .to(ToSelf)
                    .typed(system_proxy::UserBuiltinProxy)
                    .esdt_local_mint(
                        &operation_token.token_identifier,
                        operation_token.token_nonce,
                        &operation_token.token_data.amount,
                    )
                    .sync_call();
            } else {
                let arg_buffer = self.get_nft_create_args(
                    &operation_token.token_identifier,
                    &operation_token.token_nonce,
                    &operation_token.token_data,
                );

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

        self.distribute_payments(&hash_of_hashes, &operation_tuple);
    }

    fn get_nft_create_args(
        &self,
        token_identifier: &TokenIdentifier<Self::Api>,
        token_nonce: &u64,
        token_data: &StolenFromFrameworkEsdtTokenData<Self::Api>,
    ) -> ManagedArgBuffer<Self::Api> {
        let mut arg_buffer = ManagedArgBuffer::new();
        let cloned_token_data = token_data.clone();

        arg_buffer.push_arg(token_identifier);
        arg_buffer.push_arg(cloned_token_data.amount);
        arg_buffer.push_arg(cloned_token_data.name);
        arg_buffer.push_arg(cloned_token_data.royalties);
        arg_buffer.push_arg(cloned_token_data.hash);
        arg_buffer.push_arg(cloned_token_data.attributes);

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

        arg_buffer.push_arg(token_nonce);
        arg_buffer.push_arg(cloned_token_data.creator);

        arg_buffer
    }

    fn distribute_payments(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
    ) {
        let mapped_tokens: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = operation_tuple
            .operation
            .tokens
            .iter()
            .map(|token| token.into())
            .collect();

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
                            .execute(hash_of_hashes, operation_tuple),
                    )
                    .gas_for_callback(CALLBACK_GAS)
                    .register_promise();
            }
            None => {
                let own_address = self.blockchain().get_sc_address();
                let args =
                    self.get_contract_call_args(&operation_tuple.operation.to, &mapped_tokens);

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

    #[promises_callback]
    fn execute(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {}
            ManagedAsyncCallResult::Err(_) => {
                self.burn_tokens(&operation_tuple.operation);
                self.emit_transfer_failed_events(hash_of_hashes, operation_tuple);
            }
        }
    }

    fn get_contract_call_args(
        self,
        to: &ManagedAddress,
        mapped_tokens: &ManagedVec<EsdtTokenPayment<Self::Api>>,
    ) -> ManagedArgBuffer<Self::Api> {
        let mut args = ManagedArgBuffer::new();
        args.push_arg(to);
        args.push_arg(mapped_tokens.len());

        for token in mapped_tokens {
            args.push_arg(token.token_identifier);
            args.push_arg(token.token_nonce);
            args.push_arg(token.amount);
        }

        args
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
}
