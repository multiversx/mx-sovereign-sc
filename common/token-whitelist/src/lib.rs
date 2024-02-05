#![no_std]

use bls_signature::BlsSignature;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait TokenWhitelistModule:
    bls_signature::BlsSignatureModule + setup_phase::SetupPhaseModule + utils::UtilsModule
{
    /// Tokens in the whitelist can be transferred without fees
    #[endpoint(addTokensToWhitelist)]
    fn add_tokens_to_whitelist(
        &self,
        opt_signature: Option<BlsSignature<Self::Api>>,
        tokens: MultiValueEncoded<TokenIdentifier>,
    ) {
        if !self.is_setup_phase_complete() {
            self.require_caller_initiator();
            self.token_whitelist().extend(tokens);

            return;
        }

        let token_list = self.verify_items_signature(opt_signature, tokens);
        self.token_whitelist().extend(&token_list);
    }

    #[endpoint(removeTokensFromWhitelist)]
    fn remove_tokens_from_whitelist(
        &self,
        opt_signature: Option<BlsSignature<Self::Api>>,
        tokens: MultiValueEncoded<TokenIdentifier>,
    ) {
        if !self.is_setup_phase_complete() {
            self.require_caller_initiator();
            self.remove_items(&mut self.token_whitelist(), tokens);

            return;
        }

        let token_list = self.verify_items_signature(opt_signature, tokens);
        self.remove_items(&mut self.token_whitelist(), &token_list);
    }

    fn require_token_not_blacklisted(&self, token_id: &TokenIdentifier) {
        require!(
            !self.token_blacklist().contains(token_id),
            "Token blacklisted"
        );
    }

    /// Tokens in blacklist cannot be transferred
    #[endpoint(addTokensToBlacklist)]
    fn add_tokens_to_blacklist(
        &self,
        opt_signature: Option<BlsSignature<Self::Api>>,
        tokens: MultiValueEncoded<TokenIdentifier>,
    ) {
        if !self.is_setup_phase_complete() {
            self.require_caller_initiator();
            self.token_blacklist().extend(tokens);

            return;
        }

        let token_list = self.verify_items_signature(opt_signature, tokens);
        self.token_blacklist().extend(&token_list);
    }

    #[endpoint(removeTokensFromBlacklist)]
    fn remove_tokens_from_blacklist(
        &self,
        opt_signature: Option<BlsSignature<Self::Api>>,
        tokens: MultiValueEncoded<TokenIdentifier>,
    ) {
        if !self.is_setup_phase_complete() {
            self.require_caller_initiator();
            self.remove_items(&mut self.token_blacklist(), tokens);

            return;
        }

        let token_list = self.verify_items_signature(opt_signature, tokens);
        self.remove_items(&mut self.token_blacklist(), &token_list);
    }

    #[view(getTokenWhitelist)]
    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getTokenBlacklist)]
    #[storage_mapper("tokenBlacklist")]
    fn token_blacklist(&self) -> UnorderedSetMapper<TokenIdentifier>;
}
