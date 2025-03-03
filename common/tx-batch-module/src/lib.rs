#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use operation::aliases::TxNonce;

#[multiversx_sc::module]
pub trait TxBatchModule {
    fn get_and_save_next_tx_id(&self) -> TxNonce {
        self.last_tx_nonce().update(|last_tx_nonce| {
            *last_tx_nonce += 1;
            *last_tx_nonce
        })
    }

    #[storage_mapper("lastTxNonce")]
    fn last_tx_nonce(&self) -> SingleValueMapper<TxNonce>;
}
