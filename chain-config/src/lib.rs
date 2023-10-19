#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait ChainConfigContract {
    #[init]
    fn init(&self) {}

    #[endpoint]
    fn upgrade(&self) {}

    fn require_sc_address(&self, address: &ManagedAddress) {
        require!(
            !address.is_zero() && self.blockchain().is_smart_contract(address),
            "Invalid SC address"
        );
    }
}
