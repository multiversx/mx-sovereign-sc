use cross_chain::REGISTER_GAS;
use error_messages::{
    ERROR_AT_ENCODING, ESDT_SAFE_STILL_PAUSED, INVALID_PREFIX, INVALID_TYPE,
    NATIVE_TOKEN_ALREADY_REGISTERED, SETUP_PHASE_ALREADY_COMPLETED, SETUP_PHASE_NOT_COMPLETED,
    TOKEN_ALREADY_REGISTERED,
};
use multiversx_sc::types::EsdtTokenType;
use structs::{
    aliases::EventPaymentTuple, generate_hash::GenerateHash, EsdtInfo, SovTokenProperties,
};
multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD

#[multiversx_sc::module]
pub trait RegisterTokenModule:
    utils::UtilsModule
    + cross_chain::storage::CrossChainStorage
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + custom_events::CustomEventsModule
    + multiversx_sc_modules::pause::PauseModule
    + setup_phase::SetupPhaseModule
{
    #[payable("EGLD")]
    #[endpoint(registerToken)]
    fn register_token(
        &self,
        hash_of_hashes: ManagedBuffer,
        token_to_register: SovTokenProperties<Self::Api>,
    ) {
        let token_hash = token_to_register.generate_hash();
        if token_hash.is_empty() {
            self.complete_operation(&hash_of_hashes, &token_hash, Some(ERROR_AT_ENCODING.into()));
            return;
        };

        if self.is_paused() {
            self.complete_operation(
                &hash_of_hashes,
                &token_hash,
                Some(ESDT_SAFE_STILL_PAUSED.into()),
            );

            return;
        }

        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &token_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }

        self.lock_operation_hash_wrapper(&hash_of_hashes, &token_hash);

        if self.is_sov_token_id_registered(&token_to_register.token_id) {
            self.complete_operation(
                &hash_of_hashes,
                &token_hash,
                Some(TOKEN_ALREADY_REGISTERED.into()),
            );

            let tokens = self.create_event_payment_tuple(&token_to_register);

            self.deposit_event(
                &token_to_register.data.op_sender.clone(),
                &tokens,
                token_to_register.data.clone(),
            );

            return;
        }

        if !self.has_sov_prefix(&token_to_register.token_id, &self.sov_token_prefix().get()) {
            self.complete_operation(&hash_of_hashes, &token_hash, Some(INVALID_PREFIX.into()));

            return;
        }

        match token_to_register.token_type {
            EsdtTokenType::Invalid => {
                self.complete_operation(&hash_of_hashes, &token_hash, Some(INVALID_TYPE.into()))
            }
            _ => self.handle_token_issue(token_to_register, hash_of_hashes, token_hash),
        }
    }

    #[payable("EGLD")]
    #[only_owner]
    #[endpoint(registerNativeToken)]
    fn register_native_token(&self, token_ticker: ManagedBuffer, token_name: ManagedBuffer) {
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
                token_name,
                &token_ticker,
                EsdtTokenType::Fungible,
                18,
            )
            .gas(REGISTER_GAS)
            .callback(self.callbacks().native_token_issue_callback())
            .register_promise();
    }

    fn handle_token_issue(
        &self,
        args: SovTokenProperties<Self::Api>,
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
                BigUint::from(ISSUE_COST),
                token_display_name,
                token_ticker,
                token_type,
                num_decimals,
            )
            .gas(REGISTER_GAS)
            .callback(
                self.callbacks()
                    .issue_callback(&args, hash_of_hashes, token_hash),
            )
            .register_promise();
    }

    #[promises_callback]
    fn issue_callback(
        &self,
        token_to_register: &SovTokenProperties<Self::Api>,
        hash_of_hashes: ManagedBuffer,
        token_hash: ManagedBuffer,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier<Self::Api>>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(mvx_token_id) => {
                self.set_corresponding_token_ids(&token_to_register.token_id, &mvx_token_id);
                self.complete_operation(&hash_of_hashes, &token_hash, None);
            }
            ManagedAsyncCallResult::Err(error) => {
                let tokens = self.create_event_payment_tuple(token_to_register);

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
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier<Self::Api>>,
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
        sov_token_id: &TokenIdentifier,
        mvx_token_id: &TokenIdentifier,
    ) {
        self.sovereign_to_multiversx_token_id_mapper(sov_token_id)
            .set(mvx_token_id);

        self.multiversx_to_sovereign_token_id_mapper(mvx_token_id)
            .set(sov_token_id);
    }

    fn is_token_registered(&self, token_id: &TokenIdentifier, token_nonce: u64) -> bool {
        !self
            .sovereign_to_multiversx_esdt_info_mapper(token_id, token_nonce)
            .is_empty()
    }

    fn update_esdt_info_mappers(
        &self,
        sov_id: &TokenIdentifier,
        sov_nonce: u64,
        mvx_id: &TokenIdentifier,
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

    fn create_event_payment_tuple(
        &self,
        token_to_register: &SovTokenProperties<Self::Api>,
    ) -> MultiValueEncoded<Self::Api, EventPaymentTuple<Self::Api>> {
        let token_data = self.blockchain().get_esdt_token_data(
            &token_to_register.data.op_sender,
            &token_to_register.token_id,
            token_to_register.token_nonce,
        );

        MultiValueEncoded::from_iter([MultiValue3((
            token_to_register.token_id.clone(),
            token_to_register.token_nonce,
            token_data,
        ))])
    }
}
