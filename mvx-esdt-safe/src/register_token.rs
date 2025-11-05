use cross_chain::{DEFAULT_ISSUE_COST, REGISTER_GAS};
use error_messages::{
    ERROR_AT_GENERATING_OPERATION_HASH, ESDT_SAFE_STILL_PAUSED, INVALID_PREFIX_FOR_REGISTER,
    NATIVE_TOKEN_ALREADY_REGISTERED, NOT_ENOUGH_EGLD_FOR_REGISTER, SETUP_PHASE_ALREADY_COMPLETED,
    TOKEN_ALREADY_REGISTERED,
};
use multiversx_sc::{chain_core::EGLD_000000_TOKEN_IDENTIFIER, types::EsdtTokenType};
use multiversx_sc_modules::only_admin;
use structs::{
    aliases::EventPaymentTuple, generate_hash::GenerateHash, EsdtInfo, RegisterTokenOperation,
};
multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait RegisterTokenModule:
    common_utils::CommonUtilsModule
    + cross_chain::storage::CrossChainStorage
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + custom_events::CustomEventsModule
    + multiversx_sc_modules::pause::PauseModule
    + setup_phase::SetupPhaseModule
    + only_admin::OnlyAdminModule
{
    #[endpoint(registerToken)]
    fn register_sovereign_token(
        &self,
        hash_of_hashes: ManagedBuffer,
        register_token_operation: RegisterTokenOperation<Self::Api>,
    ) {
        let token_hash = register_token_operation.generate_hash();
        if let Some(lock_operation_error) = self.lock_operation_hash_wrapper(
            &hash_of_hashes,
            &token_hash,
            register_token_operation.data.op_nonce,
        ) {
            self.complete_operation(&hash_of_hashes, &token_hash, Some(lock_operation_error));
            return;
        }
        if self.is_paused() {
            self.complete_operation(
                &hash_of_hashes,
                &token_hash,
                Some(ESDT_SAFE_STILL_PAUSED.into()),
            );
            return;
        }

        if self
            .blockchain()
            .get_balance(&self.blockchain().get_sc_address())
            < DEFAULT_ISSUE_COST
        {
            self.complete_operation(
                &hash_of_hashes,
                &token_hash,
                Some(NOT_ENOUGH_EGLD_FOR_REGISTER.into()),
            );
            self.deposit_event(
                &register_token_operation.data.op_sender.clone(),
                &self.create_issue_cost_event_payment_tuple(),
                register_token_operation.data.clone(),
            );
            return;
        }
        if self.is_sov_token_id_registered(&register_token_operation.token_id) {
            self.complete_operation(
                &hash_of_hashes,
                &token_hash,
                Some(TOKEN_ALREADY_REGISTERED.into()),
            );
            self.deposit_event(
                &register_token_operation.data.op_sender.clone(),
                &self.create_issue_cost_event_payment_tuple(),
                register_token_operation.data.clone(),
            );
            return;
        }
        if !self.has_sov_prefix(
            &register_token_operation.token_id,
            &self.sov_token_prefix().get(),
        ) {
            self.complete_operation(
                &hash_of_hashes,
                &token_hash,
                Some(INVALID_PREFIX_FOR_REGISTER.into()),
            );
            self.deposit_event(
                &register_token_operation.data.op_sender.clone(),
                &self.create_issue_cost_event_payment_tuple(),
                register_token_operation.data.clone(),
            );
            return;
        }

        self.handle_token_issue(register_token_operation, hash_of_hashes, token_hash);
    }

    #[payable("EGLD")]
    #[only_admin]
    #[endpoint(registerNativeToken)]
    fn register_native_token(&self, ticker: ManagedBuffer, name: ManagedBuffer) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

        require!(
            self.native_token().is_empty(),
            NATIVE_TOKEN_ALREADY_REGISTERED
        );

        self.tx()
            .to(ESDTSystemSCAddress)
            .typed(ESDTSystemSCProxy)
            .issue_and_set_all_roles(
                self.call_value().egld().clone_value(),
                name,
                ticker,
                EsdtTokenType::Fungible,
                18,
            )
            .gas(REGISTER_GAS)
            .callback(self.callbacks().native_token_issue_callback())
            .register_promise();
    }

    fn handle_token_issue(
        &self,
        args: RegisterTokenOperation<Self::Api>,
        hash_of_hashes: ManagedBuffer,
        token_hash: ManagedBuffer,
    ) {
        let token_display_name = args.token_display_name.clone();
        let token_ticker = args.token_ticker.clone();
        let token_type = args.token_type;
        let num_decimals = args.num_decimals;

        self.tx()
            .to(ESDTSystemSCAddress)
            .typed(ESDTSystemSCProxy)
            .issue_and_set_all_roles(
                BigUint::from(DEFAULT_ISSUE_COST),
                token_display_name,
                token_ticker,
                token_type,
                num_decimals,
            )
            .gas(REGISTER_GAS)
            .callback(
                self.callbacks()
                    .register_token(&args, hash_of_hashes, token_hash),
            )
            .register_promise();
    }

    #[promises_callback]
    fn register_token(
        &self,
        token_to_register: &RegisterTokenOperation<Self::Api>,
        hash_of_hashes: ManagedBuffer,
        token_hash: ManagedBuffer,
        #[call_result] result: ManagedAsyncCallResult<EgldOrEsdtTokenIdentifier<Self::Api>>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(mvx_token_id) => {
                self.set_corresponding_token_ids(&token_to_register.token_id, &mvx_token_id);
                self.complete_operation(&hash_of_hashes, &token_hash, None);
            }
            ManagedAsyncCallResult::Err(error) => {
                let tokens = self.create_issue_cost_event_payment_tuple();

                self.deposit_event(
                    &token_to_register.data.op_sender.clone(),
                    &tokens,
                    token_to_register.data.clone(),
                );
                self.complete_operation(&hash_of_hashes, &token_hash, Some(error.err_msg));
            }
        }
    }

    #[promises_callback]
    fn native_token_issue_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<EgldOrEsdtTokenIdentifier<Self::Api>>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(native_token_id) => {
                self.native_token().set(native_token_id);
            }
            ManagedAsyncCallResult::Err(error) => {
                sc_panic!(
                    "There was an error at issuing native token: '{}'",
                    error.err_msg
                );
            }
        }
    }
    fn set_corresponding_token_ids(
        &self,
        sov_token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        mvx_token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
    ) {
        self.sovereign_to_multiversx_token_id_mapper(sov_token_id)
            .set(mvx_token_id);

        self.multiversx_to_sovereign_token_id_mapper(mvx_token_id)
            .set(sov_token_id);
    }

    fn update_esdt_info_mappers(
        &self,
        sov_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        sov_nonce: u64,
        mvx_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        new_nft_nonce: u64,
    ) {
        self.sovereign_to_multiversx_esdt_info_mapper(sov_id, sov_nonce)
            .set(EsdtInfo {
                token_identifier: mvx_id.clone(),
                token_nonce: new_nft_nonce,
            });

        self.multiversx_to_sovereign_esdt_info_mapper(mvx_id, new_nft_nonce)
            .set(EsdtInfo {
                token_identifier: sov_id.clone(),
                token_nonce: sov_nonce,
            });
    }

    #[allow(clippy::field_reassign_with_default)]
    fn create_issue_cost_event_payment_tuple(
        &self,
    ) -> MultiValueEncoded<Self::Api, EventPaymentTuple<Self::Api>> {
        let mut token_data = EsdtTokenData::default();
        token_data.amount = DEFAULT_ISSUE_COST.into();

        MultiValueEncoded::from_iter([MultiValue3((
            EGLD_000000_TOKEN_IDENTIFIER.into(),
            0u64,
            token_data,
        ))])
    }
}
