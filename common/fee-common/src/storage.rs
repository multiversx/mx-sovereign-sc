use error_messages::ONLY_ESDT_SAFE_CALLER;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeCommonStorageModule {
    fn require_caller_esdt_safe(&self) {
        let caller = self.blockchain().get_caller();
        let esdt_safe_address = self.esdt_safe_address().get();
        require!(caller == esdt_safe_address, ONLY_ESDT_SAFE_CALLER);
    }

    #[view(getUsersWhitelist)]
    #[storage_mapper("usersWhitelist")]
    fn users_whitelist(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("accFees")]
    fn accumulated_fees(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("tokensForFees")]
    fn tokens_for_fees(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[storage_mapper("esdtSafeAddress")]
    fn esdt_safe_address(&self) -> SingleValueMapper<ManagedAddress>;
}
