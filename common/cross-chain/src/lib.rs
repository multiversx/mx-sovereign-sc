#![no_std]

use operation::aliases::GasLimit;

pub mod deposit_common;
pub mod events;
pub mod storage;

pub const MAX_TRANSFERS_PER_TX: usize = 10;
pub const DEFAULT_ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD
pub const REGISTER_GAS: u64 = 60_000_000;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CrossChainCommon: crate::storage::CrossChainStorage + utils::UtilsModule {
    #[inline]
    fn require_token_not_on_blacklist(&self, token_id: &TokenIdentifier) {
        require!(
            !self
                .esdt_safe_config()
                .get()
                .token_blacklist
                .contains(token_id),
            "Token blacklisted"
        );
    }

    #[inline]
    fn is_token_whitelist_empty(&self) -> bool {
        self.esdt_safe_config().get().token_whitelist.is_empty()
    }

    #[inline]
    fn is_token_whitelisted(&self, token_id: &TokenIdentifier) -> bool {
        self.esdt_safe_config()
            .get()
            .token_whitelist
            .contains(token_id)
    }

    #[inline]
    fn require_gas_limit_under_limit(&self, gas_limit: GasLimit) {
        require!(
            gas_limit <= self.esdt_safe_config().get().max_tx_gas_limit,
            "Gas limit too high"
        );
    }

    #[inline]
    fn require_endpoint_not_banned(&self, function: &ManagedBuffer) {
        require!(
            !self
                .esdt_safe_config()
                .get()
                .banned_endpoints
                .contains(function),
            "Banned endpoint name"
        );
    }

    #[inline]
    fn is_fungible(self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::Fungible
    }

    #[inline]
    fn is_sft_or_meta(self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::SemiFungible || *token_type == EsdtTokenType::Meta
    }

    #[inline]
    fn is_nft(self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::NonFungible
    }
}
