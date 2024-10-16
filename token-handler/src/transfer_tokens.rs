use multiversx_sc::api::{ESDT_MULTI_TRANSFER_FUNC_NAME, ESDT_NFT_CREATE_FUNC_NAME};
use multiversx_sc::codec;
use multiversx_sc::err_msg;
use multiversx_sc::types::{
    system_proxy, EsdtTokenPayment, ManagedArgBuffer, MultiValueEncoded, ToSelf,
};
use multiversx_sc::types::{ManagedVec, TokenIdentifier};
use transaction::{GasLimit, OperationEsdtPayment, TransferData};

use crate::common_storage;

const TRANSACTION_GAS: GasLimit = 30_000_000;

#[multiversx_sc::module]
pub trait TransferTokensModule: common_storage::CommonStorage {
    // NOTE: will use operation.data.op_sender as well when TransferAndExecuteByUser is implemented
    #[payable("*")]
    #[endpoint(transferTokens)]
    fn transfer_tokens(
        &self,
        opt_transfer_data: Option<TransferData<Self::Api>>,
        to: ManagedAddress,
        // original_sender: ManagedAddress,
        tokens: MultiValueEncoded<OperationEsdtPayment<Self::Api>>,
    ) {
        self.require_caller_to_be_whitelisted();

        let mut output_payments = self.mint_tokens(&tokens.to_vec());
        let call_value_esdt_transfer = self.call_value().all_esdt_transfers();
        output_payments.extend(&call_value_esdt_transfer.clone_value());

        self.distribute_payments(&to, &output_payments, &opt_transfer_data);
    }

    fn distribute_payments(
        &self,
        receiver: &ManagedAddress,
        tokens: &ManagedVec<EsdtTokenPayment<Self::Api>>,
        opt_transfer_data: &Option<TransferData<Self::Api>>,
    ) {
        match &opt_transfer_data {
            Some(transfer_data) => {
                let mut args = ManagedArgBuffer::new();
                for arg in &transfer_data.args {
                    args.push_arg(arg);
                }

                self.tx()
                    .to(receiver)
                    .raw_call(transfer_data.function.clone())
                    .arguments_raw(args.clone())
                    .payment(tokens)
                    .gas(transfer_data.gas_limit)
                    .register_promise();
            }
            None => {
                let own_address = self.blockchain().get_sc_address();
                let args = self.get_contract_call_args(receiver, tokens);

                self.tx()
                    .to(own_address)
                    .raw_call(ESDT_MULTI_TRANSFER_FUNC_NAME)
                    .arguments_raw(args)
                    .gas(TRANSACTION_GAS)
                    .register_promise();
            }
        }
    }

    fn mint_tokens(
        &self,
        tokens: &ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) -> ManagedVec<EsdtTokenPayment<Self::Api>> {
        let mut output_payments: ManagedVec<Self::Api, EsdtTokenPayment> = ManagedVec::new();

        for operation_token in tokens.iter() {
            if operation_token.token_nonce == 0 {
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
                self.tx()
                    .to(ToSelf)
                    .typed(system_proxy::UserBuiltinProxy)
                    .esdt_nft_create(
                        &operation_token.token_identifier,
                        &operation_token.token_data.amount,
                        &operation_token.token_data.name,
                        &operation_token.token_data.royalties,
                        &operation_token.token_data.hash,
                        &operation_token.token_data.attributes,
                        &operation_token.token_data.uris,
                    )
                    .sync_call();
                // self.send_raw().call_local_esdt_built_in_function(
                //     self.blockchain().get_gas_left(),
                //     &ManagedBuffer::from(ESDT_NFT_CREATE_FUNC_NAME),
                //     &arg_buffer,
                // );
            }

            output_payments.push(operation_token.into());
        }

        output_payments
    }

    fn get_nft_create_args(
        &self,
        token_identifier: &TokenIdentifier<Self::Api>,
        token_nonce: &u64,
        token_data: &EsdtTokenData,
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

        arg_buffer.push_arg(cloned_token_data.token_type);
        arg_buffer.push_arg(token_nonce);
        arg_buffer.push_arg(cloned_token_data.creator);

        arg_buffer
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
