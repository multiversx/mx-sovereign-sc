#![no_std]

use multiversx_sc::storage::StorageKey;
use operation::{
    aliases::{ExtractedFeeResult, GasLimit, TxNonce},
    TransferData,
};
use proxies::fee_market_proxy::FeeMarketProxy;

pub mod events;
pub mod storage;

pub const MAX_TRANSFERS_PER_TX: usize = 10;
pub const DEFAULT_ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD
pub const REGISTER_GAS: u64 = 60_000_000;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CrossChainCommon: crate::storage::CrossChainStorage + utils::UtilsModule {
    fn match_fee_payment(
        &self,
        total_tokens_for_fees: usize,
        fees_payment: &OptionalValue<EsdtTokenPayment<Self::Api>>,
        opt_transfer_data: &Option<TransferData<<Self as ContractBase>::Api>>,
    ) {
        match fees_payment {
            OptionalValue::Some(fee) => {
                let mut gas: GasLimit = 0;

                if let Some(transfer_data) = opt_transfer_data {
                    gas = transfer_data.gas_limit;
                }

                let fee_market_address = self.fee_market_address().get();
                let caller = self.blockchain().get_caller();

                self.tx()
                    .to(fee_market_address)
                    .typed(FeeMarketProxy)
                    .subtract_fee(caller, total_tokens_for_fees, OptionalValue::Some(gas))
                    .payment(fee.clone())
                    .sync_call();
            }
            OptionalValue::None => (),
        };
    }

    fn check_and_extract_fee(&self) -> ExtractedFeeResult<Self::Api> {
        let payments = self.call_value().all_esdt_transfers().clone();

        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let fee_market_address = self.fee_market_address().get();
        let fee_enabled_mapper = SingleValueMapper::new_from_address(
            fee_market_address.clone(),
            StorageKey::from("feeEnabledFlag"),
        )
        .get();

        let opt_transfer_data = if fee_enabled_mapper {
            OptionalValue::Some(self.pop_first_payment(payments.clone()).0)
        } else {
            OptionalValue::None
        };

        MultiValue2::from((opt_transfer_data, payments))
    }

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
