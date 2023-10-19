#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait UtilsModule {
    fn require_sc_address(&self, address: &ManagedAddress) {
        require!(
            !address.is_zero() && self.blockchain().is_smart_contract(address),
            "Invalid SC address"
        );
    }
}
