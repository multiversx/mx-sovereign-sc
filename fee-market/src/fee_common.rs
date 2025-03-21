use error_messages::ONLY_ESDT_SAFE_CALLER;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CommonFeeModule {
    fn require_caller_esdt_safe(&self) {
        let caller = self.blockchain().get_caller();
        let esdt_safe_address = self.esdt_safe_address().get();
        require!(caller == esdt_safe_address, ONLY_ESDT_SAFE_CALLER);
    }

    #[storage_mapper("esdtSafeAddress")]
    fn esdt_safe_address(&self) -> SingleValueMapper<ManagedAddress>;
}
