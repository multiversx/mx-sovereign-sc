use error_messages::ONLY_ESDT_SAFE_CALLER;
use structs::fee::FeeType;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeCommonStorageModule {
    fn require_caller_esdt_safe(&self) {
        let caller = self.blockchain().get_caller();
        let esdt_safe_address = self.esdt_safe_address().get();
        require!(caller == esdt_safe_address, ONLY_ESDT_SAFE_CALLER);
    }

    fn is_fee_enabled(&self) -> bool {
        self.fee_enabled().get()
    }

    #[view(getTokenFee)]
    #[storage_mapper("tokenFee")]
    fn token_fee(
        &self,
        token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
    ) -> SingleValueMapper<FeeType<Self::Api>>;

    #[storage_mapper("feeEnabledFlag")]
    fn fee_enabled(&self) -> SingleValueMapper<bool>;

    #[view(getUsersWhitelist)]
    #[storage_mapper("usersWhitelist")]
    fn users_whitelist(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("accFees")]
    fn accumulated_fees(&self, token_id: &EsdtTokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("tokensForFees")]
    fn tokens_for_fees(&self) -> UnorderedSetMapper<EsdtTokenIdentifier>;

    #[storage_mapper("esdtSafeAddress")]
    fn esdt_safe_address(&self) -> SingleValueMapper<ManagedAddress>;
}
