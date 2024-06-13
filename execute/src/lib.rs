#![no_std]

use transaction::Operation;

mod enshrine_esdt_safe_proxy;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::contract]
pub trait Execute {
    #[init]
    fn init(&self) {}

    #[endpoint(executeOps)]
    fn execute_operation(&self, hash_of_hashes: ManagedBuffer, operation: Operation<Self::Api>) {
        let to = self.blockchain().get_sc_address();

        self.tx()
            .to(to)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .async_call_and_exit();
    }

    #[upgrade]
    fn upgrade(&self) {}
}
