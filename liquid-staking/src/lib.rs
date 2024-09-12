#![no_std]

use multiversx_sc::imports::*;

pub mod liquidity_pools;

#[multiversx_sc::contract]
pub trait LiquidStaking: liquidity_pools::LiquidityPoolModule {
    #[init]
    fn init(&self, delegation_address: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&delegation_address),
            "Provided address is not a valid Smart Contract address"
        );

        self.delegation_address().set(delegation_address);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
