#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait TokenModule {
    // endpoints - owner-only

    #[only_owner]
    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(&self, token_id: TokenIdentifier) {
        let _ = self.token_whitelist().insert(token_id);
    }

    #[only_owner]
    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        let _ = self.token_whitelist().swap_remove(&token_id);
    }

    // private

    fn require_token_in_whitelist(&self, token_id: &TokenIdentifier) {
        require!(
            self.token_whitelist().contains(token_id),
            "Token not in whitelist"
        );
    }

    fn require_local_role_set(&self, token_id: &TokenIdentifier, role: &EsdtLocalRole) {
        require!(
            self.is_local_role_set(token_id, role),
            "Must set local role first"
        );
    }

    fn is_local_role_set(&self, token_id: &TokenIdentifier, role: &EsdtLocalRole) -> bool {
        let roles = self.blockchain().get_esdt_local_roles(token_id);

        roles.has_role(role)
    }

    // storage

    #[view(getAllKnownTokens)]
    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> UnorderedSetMapper<TokenIdentifier>;
}
