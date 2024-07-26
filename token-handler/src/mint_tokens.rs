use multiversx_sc::api::ESDT_NFT_CREATE_FUNC_NAME;
use multiversx_sc::types::ManagedArgBuffer;
use multiversx_sc::types::{ManagedVec, MultiValueEncoded};
use multiversx_sc::{codec, err_msg};
use transaction::OperationEsdtPayment;

use crate::common;

#[multiversx_sc::module]
pub trait MintTokens: utils::UtilsModule + common::storage::CommonStorage {
    #[endpoint(mintTokens)]
    fn mint_tokens(&self, operation_tokens: MultiValueEncoded<OperationEsdtPayment<Self::Api>>) {
        self.require_caller_to_be_whitelisted();

        let mut output_payments: ManagedVec<Self::Api, OperationEsdtPayment<Self::Api>> =
            ManagedVec::new();

        for operation_token in operation_tokens {
            let sov_prefix = self.sov_prefix().get();
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

            // self.minted_tokens().push(&operation_token);

            output_payments.push(OperationEsdtPayment {
                token_identifier: operation_token.token_identifier,
                token_nonce: nonce,
                token_data: operation_token.token_data,
            });
        }
    }
}
