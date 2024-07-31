use multiversx_sc::api::{ESDT_MULTI_TRANSFER_FUNC_NAME, ESDT_NFT_CREATE_FUNC_NAME};
use multiversx_sc::types::{
    system_proxy, EsdtTokenPayment, ManagedArgBuffer, MultiValueEncoded, ToSelf,
};
use multiversx_sc::types::{ManagedVec, TokenIdentifier};
use multiversx_sc::{codec, err_msg};
use transaction::{GasLimit, OperationEsdtPayment, StolenFromFrameworkEsdtTokenData, TransferData};

const TRANSACTION_GAS: GasLimit = 30_000_000;

use crate::{burn_tokens, common};

#[multiversx_sc::module]
pub trait TransferTokensModule:
    utils::UtilsModule
    + common::storage::CommonStorage
    + burn_tokens::BurnTokensModule
    + tx_batch_module::TxBatchModule
{
    #[payable("*")]
    #[endpoint(transferTokens)]
    fn transfer_tokens(
        &self,
        opt_transfer_data: Option<TransferData<Self::Api>>,
        to: ManagedAddress,
        tokens: MultiValueEncoded<OperationEsdtPayment<Self::Api>>,
    ) {
        let mut output_payments: ManagedVec<Self::Api, OperationEsdtPayment<Self::Api>> =
            ManagedVec::new();
        let tokens_vec = tokens.to_vec();

        for operation_token in tokens_vec.iter() {
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

                nonce = self.call_nft_create_built_in_function(&arg_buffer);
            }

            output_payments.push(OperationEsdtPayment {
                token_identifier: operation_token.token_identifier,
                token_nonce: nonce,
                token_data: operation_token.token_data,
            });
        }

        self.distribute_payments(&tokens_vec, &opt_transfer_data, &to);
    }

    fn call_nft_create_built_in_function(&self, arg_buffer: &ManagedArgBuffer<Self::Api>) -> u64 {
        let output = self.send_raw().call_local_esdt_built_in_function(
            self.blockchain().get_gas_left(),
            &ManagedBuffer::from(ESDT_NFT_CREATE_FUNC_NAME),
            arg_buffer,
        );

        if let Some(first_result_bytes) = output.try_get(0) {
            first_result_bytes.parse_as_u64().unwrap_or_default()
        } else {
            0
        }
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
        tokens: &ManagedVec<OperationEsdtPayment<Self::Api>>,
        opt_transfer_data: &Option<TransferData<Self::Api>>,
        to: &ManagedAddress,
    ) {
        let mapped_tokens: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> =
            tokens.iter().map(|token| token.into()).collect();

        match &opt_transfer_data {
            Some(transfer_data) => {
                let mut args = ManagedArgBuffer::new();
                for arg in &transfer_data.args {
                    args.push_arg(arg);
                }

                self.tx()
                    .to(to)
                    .raw_call(transfer_data.function.clone())
                    .arguments_raw(args.clone())
                    .multi_esdt(mapped_tokens.clone())
                    .gas(transfer_data.gas_limit)
                    .register_promise();
            }
            None => {
                let own_address = self.blockchain().get_sc_address();
                let args = self.get_contract_call_args(to, &mapped_tokens);

                self.tx()
                    .to(own_address)
                    .raw_call(ESDT_MULTI_TRANSFER_FUNC_NAME)
                    .arguments_raw(args)
                    .gas(TRANSACTION_GAS)
                    .register_promise();
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
}
