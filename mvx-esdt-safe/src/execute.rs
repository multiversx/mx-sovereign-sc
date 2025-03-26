use error_messages::ESDT_SAFE_STILL_PAUSED;
use multiversx_sc::api::ESDT_MULTI_TRANSFER_FUNC_NAME;
use structs::{
    aliases::GasLimit,
    operation::{Operation, OperationData, OperationEsdtPayment, OperationTuple},
};

multiversx_sc::imports!();
const CALLBACK_GAS: GasLimit = 10_000_000; // Increase if not enough
const ESDT_TRANSACTION_GAS: GasLimit = 5_000_000;

#[multiversx_sc::module]
pub trait ExecuteModule:
    crate::bridging_mechanism::BridgingMechanism
    + crate::register_token::RegisterTokenModule
    + utils::UtilsModule
    + cross_chain::events::EventsModule
    + cross_chain::storage::CrossChainStorage
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + multiversx_sc_modules::pause::PauseModule
    + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[endpoint(executeBridgeOps)]
    fn execute_operations(&self, hash_of_hashes: ManagedBuffer, operation: Operation<Self::Api>) {
        require!(self.not_paused(), ESDT_SAFE_STILL_PAUSED);

        let operation_hash = self.calculate_operation_hash(&operation);

        self.lock_operation_hash(&operation_hash, &hash_of_hashes);

        let operation_tuple = OperationTuple {
            op_hash: operation_hash,
            operation: operation.clone(),
        };

        let minted_operation_tokens =
            self.mint_tokens(&hash_of_hashes, &operation_tuple, &operation.tokens);

        self.distribute_payments(&hash_of_hashes, &operation_tuple, &minted_operation_tokens);
    }

    fn mint_tokens(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
        operation_tokens: &ManagedVec<OperationEsdtPayment<Self::Api>>,
    ) -> ManagedVec<OperationEsdtPayment<Self::Api>> {
        let mut output_payments = ManagedVec::new();

        for operation_token in operation_tokens.iter() {
            if let Some(mvx_token_id) = self.resolve_mvx_token_id(&operation_token) {
                if !self.validate_and_burn_execute_payment(
                    &operation_token,
                    hash_of_hashes,
                    operation_tuple,
                ) {
                    return ManagedVec::new();
                }

                let nft_nonce = self.esdt_create_and_update_mapper(&mvx_token_id, &operation_token);

                output_payments.push(OperationEsdtPayment::new(
                    mvx_token_id,
                    nft_nonce,
                    operation_token.token_data.clone(),
                ));
            } else {
                output_payments.push(operation_token.clone());
            }
        }

        output_payments
    }

    fn validate_and_burn_execute_payment(
        &self,
        payment: &OperationEsdtPayment<Self::Api>,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
    ) -> bool {
        if self.is_fungible(&payment.token_data.token_type)
            && self
                .burn_mechanism_tokens()
                .contains(&payment.token_identifier)
        {
            let deposited_token_amount_mapper =
                self.deposited_tokens_amount(&payment.token_identifier);

            let deposited_amount = deposited_token_amount_mapper.get();

            if !self
                .is_execute_burn_payment_amount_valid(&deposited_amount, &payment.token_data.amount)
            {
                self.emit_transfer_failed_events(hash_of_hashes, operation_tuple);
                self.remove_executed_hash(hash_of_hashes, &operation_tuple.op_hash);

                return false;
            }

            deposited_token_amount_mapper
                .update(|amount| *amount -= payment.token_data.amount.clone());

            self.tx()
                .to(ToSelf)
                .typed(UserBuiltinProxy)
                .esdt_local_mint(&payment.token_identifier, 0, &payment.token_data.amount)
                .sync_call();
        }

        true
    }

    fn is_execute_burn_payment_amount_valid(
        &self,
        deposited_amount: &BigUint<Self::Api>,
        payment_amount: &BigUint<Self::Api>,
    ) -> bool {
        if payment_amount > deposited_amount {
            return false;
        }

        true
    }

    fn resolve_mvx_token_id(
        &self,
        operation_token: &OperationEsdtPayment<Self::Api>,
    ) -> Option<TokenIdentifier<Self::Api>> {
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

    fn esdt_create_and_update_mapper(
        self,
        mvx_token_id: &TokenIdentifier<Self::Api>,
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
        if tokens_list.is_empty() {
            return;
        }

        let mapped_tokens: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = tokens_list
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
                    .payment(&mapped_tokens)
                    .gas(transfer_data.gas_limit)
                    .callback(
                        <Self as ExecuteModule>::callbacks(self)
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
                        <Self as ExecuteModule>::callbacks(self)
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

        self.remove_executed_hash(hash_of_hashes, &operation_tuple.op_hash);
    }

    fn emit_transfer_failed_events(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_tuple: &OperationTuple<Self::Api>,
    ) {
        self.execute_bridge_operation_event(hash_of_hashes, &operation_tuple.op_hash);

        for operation_token in &operation_tuple.operation.tokens {
            self.burn_failed_transfer_token(&operation_token);
        }

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

    fn burn_failed_transfer_token(&self, operation_token: &OperationEsdtPayment<Self::Api>) {
        let sov_to_mvx_mapper =
            self.sovereign_to_multiversx_token_id_mapper(&operation_token.token_identifier);

        if sov_to_mvx_mapper.is_empty() {
            return;
        }

        let mvx_token_id = sov_to_mvx_mapper.get();
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

    fn get_mvx_nonce_from_mapper(self, token_id: &TokenIdentifier, nonce: u64) -> u64 {
        let esdt_info_mapper = self.sovereign_to_multiversx_esdt_info_mapper(token_id, nonce);
        if esdt_info_mapper.is_empty() {
            return 0;
        }
        esdt_info_mapper.get().token_nonce
    }
}
