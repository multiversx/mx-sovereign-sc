#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait TestingSc {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[payable("*")]
    #[endpoint]
    fn hello(&self, value: BigUint) {
        require!(value > BigUint::zero(), "Value should be greater than 0")
    }

    #[endpoint]
    fn send_tokens(&self, token_id: TokenIdentifier, nonce: u64, amount: BigUint) {
        let self_address = self.blockchain().get_sc_address();
        let receiver = self.blockchain().get_caller();

        let sc_balance = self
            .blockchain()
            .get_esdt_balance(&self_address, &token_id, nonce);
        require!(sc_balance >= amount, "Insufficient balance");

        self.send()
            .direct_esdt(&receiver, &token_id, nonce, &amount);
    }
}
