#![no_std]

use operation::aliases::{GasLimit, TxNonce};

pub mod events;
pub mod storage;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CrossChainCommon: crate::storage::CrossChainStorage {
    #[inline]
    fn require_token_not_on_blacklist(&self, token_id: &TokenIdentifier) {
        require!(
            self.cross_chain_config()
                .get()
                .token_blacklist
                .contains(token_id),
            "Token blacklisted"
        );
    }

    #[inline]
    fn is_token_whitelist_empty(&self) -> bool {
        self.cross_chain_config().get().token_whitelist.is_empty()
    }

    #[inline]
    fn is_token_whitelisted(&self, token_id: &TokenIdentifier) -> bool {
        self.cross_chain_config()
            .get()
            .token_whitelist
            .contains(token_id)
    }

    #[inline]
    fn require_gas_limit_under_limit(&self, gas_limit: GasLimit) {
        require!(
            gas_limit <= self.cross_chain_config().get().max_tx_gas_limit,
            "Gas limit too high"
        );
    }

    #[inline]
    fn require_endpoint_not_banned(&self, function: &ManagedBuffer) {
        require!(
            !self
                .cross_chain_config()
                .get()
                .banned_endpoints
                .contains(function),
            "Banned endpoint name"
        );
    }

    #[inline]
    fn get_and_save_next_tx_id(&self) -> TxNonce {
        self.last_tx_nonce().update(|last_tx_nonce| {
            *last_tx_nonce += 1;
            *last_tx_nonce
        })
    }
}
