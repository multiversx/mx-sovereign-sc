use cross_chain::REGISTER_GAS;
use error_messages::{
    CANNOT_REGISTER_TOKEN, ERROR_AT_ENCODING, ESDT_SAFE_STILL_PAUSED, INVALID_TYPE,
    NATIVE_TOKEN_ALREADY_REGISTERED,
};
use multiversx_sc::types::EsdtTokenType;
use structs::{generate_hash::GenerateHash, EsdtInfo, IssueEsdtArgs, UnregisteredTokenProperties};
multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait RegisterTokenModule:
    utils::UtilsModule
    + cross_chain::storage::CrossChainStorage
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + custom_events::CustomEventsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[payable("EGLD")]
    #[endpoint(registerToken)]
    fn register_token(
        &self,
        hash_of_hashes: ManagedBuffer,
        token_to_register: UnregisteredTokenProperties<Self::Api>,
    ) {
        let token_hash = token_to_register.generate_hash();
        if token_hash.is_empty() {
            self.complete_operation(&hash_of_hashes, &token_hash, Some(ERROR_AT_ENCODING.into()));
        };
        if self.is_paused() {
            self.complete_operation(
                &hash_of_hashes,
                &token_hash,
                Some(ESDT_SAFE_STILL_PAUSED.into()),
            );
        }

        self.require_sov_token_id_not_registered(&token_to_register.token_id);

        // if !self.is_token_registered(&token_to_register.token_id, token_to_register) {}

        if self.has_sov_prefix(&token_to_register.token_id, &self.sov_token_prefix().get()) {}
        let issue_cost = self.call_value().egld().clone_value();

        // match token_type {
        //     EsdtTokenType::Invalid => sc_panic!(INVALID_TYPE),
        //     _ => self.handle_token_issue(IssueEsdtArgs {
        //         sov_token_id: sov_token_id.clone(),
        //         issue_cost,
        //         token_display_name,
        //         token_ticker,
        //         token_type,
        //         num_decimals,
        //     }),
        // }
    }

    #[payable("EGLD")]
    #[only_owner]
    #[endpoint(registerNativeToken)]
    fn register_native_token(&self, token_ticker: ManagedBuffer, token_name: ManagedBuffer) {
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

    fn handle_token_issue(&self, args: IssueEsdtArgs<Self::Api>) {
        self.tx()
            .to(ESDTSystemSCAddress)
            .typed(ESDTSystemSCProxy)
            .issue_and_set_all_roles(
                args.issue_cost,
                args.token_display_name,
                args.token_ticker,
                args.token_type,
                args.num_decimals,
            )
            .gas(REGISTER_GAS)
            .callback(self.callbacks().issue_callback(&args.sov_token_id))
            .register_promise();
    }

    #[promises_callback]
    fn issue_callback(
        &self,
        sov_token_id: &TokenIdentifier,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier<Self::Api>>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(mvx_token_id) => {
                self.set_corresponding_token_ids(sov_token_id, &mvx_token_id);
            }
            ManagedAsyncCallResult::Err(error) => {
                sc_panic!("There was an error at issuing token: '{}'", error.err_msg);
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
}
