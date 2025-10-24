use error_messages::{
    BURN_ESDT_FAILED, CREATE_ESDT_FAILED, DEPOSIT_AMOUNT_NOT_ENOUGH,
    ERROR_AT_GENERATING_OPERATION_HASH, ESDT_SAFE_STILL_PAUSED, MINT_ESDT_FAILED,
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

        let minted_operation_tokens = match self.process_operation_payments(&operation_tuple) {
            Ok(tokens) => tokens,
            Err(err_msg) => {
                self.complete_operation(&hash_of_hashes, &operation_tuple.op_hash, Some(err_msg));
                return;
            }
        };
        self.distribute_payments(&hash_of_hashes, &operation_tuple, &minted_operation_tokens);
    }

    fn process_operation_payments(
        &self,
        operation_tuple: &OperationTuple<Self::Api>,
    ) -> Result<ManagedVec<OperationEsdtPayment<Self::Api>>, ManagedBuffer> {
        let mut output_payments = ManagedVec::new();

        for operation_token in operation_tuple.operation.tokens.iter() {
            let processing_result = match self.get_mvx_token_id(&operation_token) {
                Some(mvx_token_id) => self.process_resolved_token(&mvx_token_id, &operation_token),
                None => self.process_unresolved_token(&operation_token),
            };

            match processing_result {
                Ok(payment) => output_payments.push(payment),
                Err(err_msg) => {
                    let refund_result =
                        self.refund_transfers(&output_payments, &operation_tuple.operation);
                    return Err(self.merge_error_if_any(err_msg, refund_result));
                }
            };
        }

        Ok(output_payments)
    }

    fn process_resolved_token(
        &self,
        mvx_token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) -> Result<OperationEsdtPayment<Self::Api>, ManagedBuffer> {
        let mut nonce: u64 = 0;
        if self.is_fungible(&operation_token.token_data.token_type) {
            self.mint_fungible_token(mvx_token_id, &operation_token.token_data.amount)?;
        } else {
            nonce = self.esdt_create_and_update_mapper(mvx_token_id, operation_token)?;
        }

        Ok(OperationEsdtPayment::new(
            mvx_token_id.clone(),
            nonce,
            operation_token.token_data.clone(),
        ))
    }

    fn process_unresolved_token(
        &self,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) -> Result<OperationEsdtPayment<Self::Api>, ManagedBuffer> {
        if self.is_burn_mechanism_set(operation_token) {
            let deposited_mapper = self.deposited_tokens_amount(&operation_token.token_identifier);
            let deposited_amount = deposited_mapper.get();

            if operation_token.token_data.amount > deposited_amount {
                return Err(DEPOSIT_AMOUNT_NOT_ENOUGH.into());
            }

            // Mint fungible tokens first; only deduct deposited amount after success
            self.mint_fungible_token(
                &operation_token.token_identifier,
                &operation_token.token_data.amount,
            )?;
            deposited_mapper.update(|amount| *amount -= operation_token.token_data.amount.clone());
        }

        Ok(operation_token.clone())
    }

    fn mint_fungible_token(
        &self,
        token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        amount: &BigUint,
    ) -> Result<(), ManagedBuffer> {
        let esdt_token_id = token_id.clone().unwrap_esdt();
        let result = self
            .tx()
            .to(ToSelf)
            .typed(UserBuiltinProxy)
            .esdt_local_mint(esdt_token_id.clone(), 0, amount)
            .returns(ReturnsHandledOrError::new())
            .sync_call_fallible();

        match result {
            Ok(_) => Ok(()),
            Err(error_code) => Err(self.format_error(MINT_ESDT_FAILED, esdt_token_id, error_code)),
        }
    }

    fn esdt_create_and_update_mapper(
        &self,
        mvx_token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) -> Result<u64, ManagedBuffer> {
        let mut nonce = 0;
        let current_token_type_ref = &operation_token.token_data.token_type;

        if self.is_sft_or_meta(current_token_type_ref) {
            nonce = self.get_mvx_nonce_from_mapper(
                &operation_token.token_identifier,
                operation_token.token_nonce,
            )
        }

        if nonce == 0 {
            let new_nonce = self.create_esdt(mvx_token_id, &operation_token.token_data)?;
            self.update_esdt_info_mappers(
                &operation_token.token_identifier,
                operation_token.token_nonce,
                mvx_token_id,
                new_nonce,
            );
            return Ok(new_nonce);
        }

        self.add_esdt_supply(mvx_token_id, nonce, &operation_token.token_data.amount)?;
        Ok(nonce)
    }

    fn add_esdt_supply(
        &self,
        token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        nonce: u64,
        amount: &BigUint,
    ) -> Result<(), ManagedBuffer> {
        let esdt_token_id = token_id.clone().unwrap_esdt();
        let result = self
            .tx()
            .to(ToSelf)
            .typed(UserBuiltinProxy)
            .esdt_local_mint(esdt_token_id.clone(), nonce, amount)
            .returns(ReturnsHandledOrError::new())
            .sync_call_fallible();

        match result {
            Ok(_) => Ok(()),
            Err(error_code) => Err(self.format_error(MINT_ESDT_FAILED, esdt_token_id, error_code)),
        }
    }

    fn create_esdt(
        &self,
        token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        token_data: &EsdtTokenData<Self::Api>,
    ) -> Result<u64, ManagedBuffer> {
        let mut amount = token_data.amount.clone();
        if self.is_sft_or_meta(&token_data.token_type) {
            amount += BigUint::from(1u32);
        }

        let esdt_token_id = token_id.clone().unwrap_esdt();
        let result = self
            .tx()
            .to(ToSelf)
            .typed(UserBuiltinProxy)
            .esdt_nft_create(
                esdt_token_id.clone(),
                &amount,
                &token_data.name,
                &token_data.royalties,
                &token_data.hash,
                &token_data.attributes,
                &token_data.uris,
            )
            .returns(ReturnsHandledOrError::new().returns(ReturnsResult))
            .sync_call_fallible();

        match result {
            Ok(nonce) => Ok(nonce),
            Err(error_code) => {
                Err(self.format_error(CREATE_ESDT_FAILED, esdt_token_id, error_code))
            }
        }
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
                let refund_result =
                    self.refund_transfers(output_payments, &operation_tuple.operation);
                let error_message = self.merge_error_if_any(err.err_msg, refund_result);

                self.complete_operation(
                    hash_of_hashes,
                    &operation_tuple.op_hash,
                    Some(error_message),
                );
            }
        }
    }

    fn refund_transfers(
        &self,
        output_payments: &ManagedVec<OperationEsdtPayment<Self::Api>>,
        operation: &Operation<Self::Api>,
    ) -> Result<(), ManagedBuffer> {
        if output_payments.is_empty() {
            return Ok(());
        }

        let mut burn_errors = ManagedVec::new();

        for i in 0..output_payments.len() {
            match self.burn_failed_transfer_token(&output_payments.get(i), &operation.tokens.get(i))
            {
                Ok(_) => {
                    if self.is_burn_mechanism_set(&output_payments.get(i)) {
                        self.deposited_tokens_amount(&output_payments.get(i).token_identifier)
                            .update(|amount| {
                                *amount += output_payments.get(i).token_data.amount.clone()
                            });
                    }
                }
                Err(err_msg) => {
                    burn_errors.push(err_msg);
                }
            }
        }

        let sc_address = self.blockchain().get_sc_address();
        let tx_nonce = self.get_current_and_increment_tx_nonce();
        self.deposit_event(
            &operation.data.op_sender,
            &operation.map_tokens_to_multi_value_encoded(),
            OperationData::new(tx_nonce, sc_address.clone(), None),
        );

        if !burn_errors.is_empty() {
            return Err(self.combine_error_messages(&burn_errors));
        }
        Ok(())
    }

    fn burn_failed_transfer_token(
        &self,
        output_payment: &OperationEsdtPayment<Self::Api>,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) -> Result<(), ManagedBuffer> {
        let mvx_to_sov_mapper =
            self.multiversx_to_sovereign_token_id_mapper(&output_payment.token_identifier);
        if mvx_to_sov_mapper.is_empty() && !self.is_native_token(&output_payment.token_identifier) {
            return Ok(());
        }

        let esdt_token_id = output_payment.token_identifier.clone().unwrap_esdt();
        let result = self
            .tx()
            .to(ToSelf)
            .typed(UserBuiltinProxy)
            .esdt_local_burn(
                esdt_token_id.clone(),
                output_payment.token_nonce,
                &output_payment.token_data.amount,
            )
            .returns(ReturnsHandledOrError::new())
            .sync_call_fallible();

        match result {
            Ok(_) => {
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

                Ok(())
            }
            Err(error_code) => Err(self.format_error(BURN_ESDT_FAILED, esdt_token_id, error_code)),
        }
    }

    fn merge_error_if_any(
        &self,
        outer_error: ManagedBuffer,
        result: Result<(), ManagedBuffer>,
    ) -> ManagedBuffer {
        match result {
            Ok(()) => outer_error,
            Err(refund_err) => {
                let mut errors: ManagedVec<Self::Api, ManagedBuffer> = ManagedVec::new();
                errors.push(outer_error);
                errors.push(refund_err);
                self.combine_error_messages(&errors)
            }
        }
    }

    fn combine_error_messages(
        &self,
        errors: &ManagedVec<Self::Api, ManagedBuffer>,
    ) -> ManagedBuffer {
        let newline: ManagedBuffer = "\n".into();
        let mut aggregated = ManagedBuffer::new();

        for i in 0..errors.len() {
            let error_message = errors.get(i);
            aggregated.append(&error_message);
            if i + 1 < errors.len() {
                aggregated.append(&newline);
            }
        }

        aggregated
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

    #[inline]
    fn format_error(
        &self,
        error: &str,
        token_id: TokenIdentifier,
        error_code: u32,
    ) -> ManagedBuffer {
        let prefix: ManagedBuffer = error.into();
        let error_message = sc_format!("{} {}; error code: {}", prefix, token_id, error_code);

        error_message
    }

    #[inline]
    fn is_burn_mechanism_set(&self, operation_token: &OperationEsdtPayment<Self::Api>) -> bool {
        self.is_fungible(&operation_token.token_data.token_type)
            && self
                .burn_mechanism_tokens()
                .contains(&operation_token.token_identifier)
    }
}
