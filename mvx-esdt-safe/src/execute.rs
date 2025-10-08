use error_messages::{
    DEPOSIT_AMOUNT_NOT_ENOUGH, ERROR_AT_GENERATING_OPERATION_HASH, ESDT_SAFE_STILL_PAUSED,
    SETUP_PHASE_NOT_COMPLETED,
};
use multiversx_sc_modules::only_admin;
use structs::{
    aliases::GasLimit,
    generate_hash::GenerateHash,
    operation::{Operation, OperationData, OperationEsdtPayment, OperationTuple},
};

multiversx_sc::imports!();
const CALLBACK_GAS: GasLimit = 10_000_000; // Increase if not enough
const ESDT_TRANSACTION_GAS: GasLimit = 5_000_000;

#[multiversx_sc::module]
pub trait ExecuteModule:
    crate::bridging_mechanism::BridgingMechanism
    + crate::register_token::RegisterTokenModule
    + common_utils::CommonUtilsModule
    + setup_phase::SetupPhaseModule
    + custom_events::CustomEventsModule
    + cross_chain::storage::CrossChainStorage
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + multiversx_sc_modules::pause::PauseModule
    + only_admin::OnlyAdminModule
{
    #[endpoint(executeBridgeOps)]
    fn execute_operations(&self, hash_of_hashes: ManagedBuffer, operation: Operation<Self::Api>) {
        let operation_hash = operation.generate_hash();
        if operation_hash.is_empty() {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(ERROR_AT_GENERATING_OPERATION_HASH.into()),
            );
            return;
        };
        if self.is_paused() {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(ESDT_SAFE_STILL_PAUSED.into()),
            );
            return;
        }
        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }
        if let Some(lock_operation_error) = self.lock_operation_hash_wrapper(
            &hash_of_hashes,
            &operation_hash,
            operation.data.op_nonce,
        ) {
            self.complete_operation(&hash_of_hashes, &operation_hash, Some(lock_operation_error));
            return;
        }

        let operation_tuple = OperationTuple {
            op_hash: operation_hash,
            operation: operation.clone(),
        };

        if operation.tokens.is_empty() {
            self.execute_sc_call(&hash_of_hashes, &operation_tuple);

            return;
        }

        if let Some(minted_operation_tokens) = self.mint_tokens(&hash_of_hashes, &operation_tuple) {
            self.distribute_payments(&hash_of_hashes, &operation_tuple, &minted_operation_tokens);
        }
    }

    fn mint_tokens(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
    ) -> Option<ManagedVec<OperationEsdtPayment<Self::Api>>> {
        let mut output_payments = ManagedVec::new();

        for operation_token in operation_tuple.operation.tokens.iter() {
            match self.get_mvx_token_id(&operation_token) {
                Some(mvx_token_id) => {
                    let payment = self.process_resolved_token(&mvx_token_id, &operation_token);
                    output_payments.push(payment);
                }
                None => {
                    if let Some(payment) = self.process_unresolved_token(
                        hash_of_hashes,
                        operation_tuple,
                        &operation_token,
                    ) {
                        output_payments.push(payment);
                    } else {
                        self.refund_transfers(&output_payments, &operation_tuple.operation);
                        return None;
                    }
                }
            }
        }

        Some(output_payments)
    }

    fn process_resolved_token(
        &self,
        mvx_token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) -> OperationEsdtPayment<Self::Api> {
        if self.is_fungible(&operation_token.token_data.token_type) {
            self.mint_fungible_token(mvx_token_id, &operation_token.token_data.amount);
            OperationEsdtPayment::new(mvx_token_id.clone(), 0, operation_token.token_data.clone())
        } else {
            let nft_nonce = self.esdt_create_and_update_mapper(mvx_token_id, operation_token);
            OperationEsdtPayment::new(
                mvx_token_id.clone(),
                nft_nonce,
                operation_token.token_data.clone(),
            )
        }
    }

    fn process_unresolved_token(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) -> Option<OperationEsdtPayment<Self::Api>> {
        if self.is_fungible(&operation_token.token_data.token_type)
            && self
                .burn_mechanism_tokens()
                .contains(&operation_token.token_identifier)
        {
            let deposited_mapper = self.deposited_tokens_amount(&operation_token.token_identifier);
            let deposited_amount = deposited_mapper.get();

            if operation_token.token_data.amount > deposited_amount {
                self.complete_operation(
                    hash_of_hashes,
                    &operation_tuple.op_hash,
                    Some(DEPOSIT_AMOUNT_NOT_ENOUGH.into()),
                );
                return None;
            }

            deposited_mapper.update(|amount| *amount -= operation_token.token_data.amount.clone());
            self.mint_fungible_token(
                &operation_token.token_identifier,
                &operation_token.token_data.amount,
            );
        }

        Some(operation_token.clone())
    }

    fn mint_fungible_token(
        &self,
        token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        amount: &BigUint,
    ) {
        self.tx()
            .to(ToSelf)
            .typed(UserBuiltinProxy)
            .esdt_local_mint(token_id.clone().unwrap_esdt(), 0, amount)
            .sync_call();
    }

    fn esdt_create_and_update_mapper(
        &self,
        mvx_token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) -> u64 {
        let mut nonce = 0;
        let current_token_type_ref = &operation_token.token_data.token_type;

        if self.is_sft_or_meta(current_token_type_ref) {
            nonce = self.get_mvx_nonce_from_mapper(
                &operation_token.token_identifier,
                operation_token.token_nonce,
            )
        }

        if nonce == 0 {
            nonce = self.mint_nft_tx(mvx_token_id, &operation_token.token_data);

            self.update_esdt_info_mappers(
                &operation_token.token_identifier,
                operation_token.token_nonce,
                mvx_token_id,
                nonce,
            );
        } else {
            self.tx()
                .to(ToSelf)
                .typed(system_proxy::UserBuiltinProxy)
                .esdt_local_mint(
                    mvx_token_id.clone().unwrap_esdt(),
                    nonce,
                    &operation_token.token_data.amount,
                )
                .sync_call();
        }

        nonce
    }

    fn mint_nft_tx(
        &self,
        mvx_token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
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
                mvx_token_id.clone().unwrap_esdt(),
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

    fn distribute_payments(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
        output_payments: &ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) {
        let payment_tokens: ManagedVec<Self::Api, EgldOrEsdtTokenPayment<Self::Api>> =
            output_payments
                .iter()
                .map(|token| token.clone().into())
                .collect();

        match &operation_tuple.operation.data.opt_transfer_data {
            Some(transfer_data) => {
                let args = ManagedArgBuffer::from(transfer_data.args.clone());

                self.tx()
                    .to(&operation_tuple.operation.to)
                    .raw_call(transfer_data.function.clone())
                    .arguments_raw(args)
                    .payment(&payment_tokens)
                    .gas(transfer_data.gas_limit)
                    .callback(<Self as ExecuteModule>::callbacks(self).execute(
                        hash_of_hashes,
                        output_payments,
                        operation_tuple,
                    ))
                    .gas_for_callback(CALLBACK_GAS)
                    .register_promise();
            }
            None => {
                self.tx()
                    .to(&operation_tuple.operation.to)
                    .payment(&payment_tokens)
                    .gas(ESDT_TRANSACTION_GAS)
                    .callback(<Self as ExecuteModule>::callbacks(self).execute(
                        hash_of_hashes,
                        output_payments,
                        operation_tuple,
                    ))
                    .gas_for_callback(CALLBACK_GAS)
                    .register_promise();
            }
        }
    }

    fn execute_sc_call(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
    ) {
        let transfer_data = operation_tuple
            .operation
            .data
            .opt_transfer_data
            .as_ref()
            .unwrap();
        let args = ManagedArgBuffer::from(transfer_data.args.clone());

        self.tx()
            .to(&operation_tuple.operation.to)
            .raw_call(transfer_data.function.clone())
            .arguments_raw(args)
            .gas(transfer_data.gas_limit)
            .callback(<Self as ExecuteModule>::callbacks(self).execute(
                hash_of_hashes,
                &ManagedVec::new(),
                operation_tuple,
            ))
            .gas_for_callback(CALLBACK_GAS)
            .register_promise();
    }

    #[promises_callback]
    fn execute(
        &self,
        hash_of_hashes: &ManagedBuffer,
        output_payments: &ManagedVec<OperationEsdtPayment<Self::Api>>,
        operation_tuple: &OperationTuple<Self::Api>,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                self.complete_operation(hash_of_hashes, &operation_tuple.op_hash, None);
            }
            ManagedAsyncCallResult::Err(err) => {
                self.complete_operation(
                    hash_of_hashes,
                    &operation_tuple.op_hash,
                    Some(err.err_msg),
                );
                self.refund_transfers(output_payments, &operation_tuple.operation);
            }
        }
    }

    fn refund_transfers(
        &self,
        output_payments: &ManagedVec<OperationEsdtPayment<Self::Api>>,
        operation: &Operation<Self::Api>,
    ) {
        if output_payments.is_empty() {
            return;
        }

        for i in 0..output_payments.len() {
            self.burn_failed_transfer_token(&output_payments.get(i), &operation.tokens.get(i));
        }

        let sc_address = self.blockchain().get_sc_address();
        let tx_nonce = self.get_current_and_increment_tx_nonce();
        self.deposit_event(
            &operation.data.op_sender,
            &operation.map_tokens_to_multi_value_encoded(),
            OperationData::new(tx_nonce, sc_address.clone(), None),
        );
    }

    fn burn_failed_transfer_token(
        &self,
        output_payment: &OperationEsdtPayment<Self::Api>,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) {
        let mvx_to_sov_mapper =
            self.multiversx_to_sovereign_token_id_mapper(&output_payment.token_identifier);
        if mvx_to_sov_mapper.is_empty() && !self.is_native_token(&output_payment.token_identifier) {
            return;
        }

        if self.is_nft(&operation_token.token_data.token_type) {
            self.clear_mvx_to_sov_esdt_info_mapper(
                &output_payment.token_identifier,
                output_payment.token_nonce,
            );
            self.clear_sov_to_mvx_esdt_info_mapper(
                &operation_token.token_identifier,
                operation_token.token_nonce,
            );
        }

        self.tx()
            .to(ToSelf)
            .typed(UserBuiltinProxy)
            .esdt_local_burn(
                output_payment.token_identifier.clone().unwrap_esdt(),
                output_payment.token_nonce,
                &output_payment.token_data.amount,
            )
            .sync_call();
    }

    fn get_mvx_token_id(
        &self,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) -> Option<EgldOrEsdtTokenIdentifier<Self::Api>> {
        let sov_to_mvx_mapper =
            self.sovereign_to_multiversx_token_id_mapper(&operation_token.token_identifier);

        if !sov_to_mvx_mapper.is_empty() {
            return Some(sov_to_mvx_mapper.get());
        }

        if self.is_native_token(&operation_token.token_identifier) {
            Some(operation_token.token_identifier.clone())
        } else {
            None
        }
    }

    fn get_mvx_nonce_from_mapper(
        &self,
        token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        nonce: u64,
    ) -> u64 {
        let esdt_info_mapper = self.sovereign_to_multiversx_esdt_info_mapper(token_id, nonce);
        if esdt_info_mapper.is_empty() {
            return 0;
        }
        esdt_info_mapper.get().token_nonce
    }
}
