#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;
use proxies::mvx_esdt_safe_proxy::MvxEsdtSafeProxy;

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
    fn view_storage(&self, wanted_address: ManagedAddress) {
        self.tx()
            .to(&wanted_address)
            .typed(MvxEsdtSafeProxy)
            .native_token()
            .sync_call();
    }

    #[endpoint]
    fn view_storage_async(&self, wanted_address: ManagedAddress) {
        self.tx()
            .to(&wanted_address)
            .typed(MvxEsdtSafeProxy)
            .native_token()
            .async_call_and_exit();
    }

    #[endpoint]
    fn view_storage_sync(&self, wanted_address: ManagedAddress) {
        self.tx()
            .to(&wanted_address)
            .typed(MvxEsdtSafeProxy)
            .native_token()
            .sync_call();
    }

    #[endpoint]
    fn send_tokens(&self, token_id: EgldOrEsdtTokenIdentifier, nonce: u64, amount: BigUint) {
        let self_address = self.blockchain().get_sc_address();
        let receiver = self.blockchain().get_caller();

        let sc_balance = self.blockchain().get_esdt_balance(
            &self_address,
            &token_id.clone().unwrap_esdt(),
            nonce,
        );
        require!(sc_balance >= amount, "Insufficient balance");

        self.send()
            .direct_esdt(&receiver, &token_id.unwrap_esdt(), nonce, &amount);
    }
}
