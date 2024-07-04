#![no_std]

use multiversx_sc::imports::*;

pub mod common;
pub mod enshrine_esdt_safe_proxy;
pub mod from_sovereign;
pub mod to_sovereign;

#[multiversx_sc::contract]
pub trait EnshrineEsdtSafe:
    to_sovereign::create_tx::CreateTxModule
    + to_sovereign::events::EventsModule
    + bls_signature::BlsSignatureModule
    + from_sovereign::events::EventsModule
    + from_sovereign::transfer_tokens::TransferTokensModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + setup_phase::SetupPhaseModule
    + token_whitelist::TokenWhitelistModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + common::storage::CommonStorage
{
    #[init]
    fn init(
        &self,
        is_sovereign_chain: bool,
        wegld_identifier: Option<ManagedBuffer>,
        sov_token_prefix: Option<ManagedBuffer>,
    ) {
        self.is_sovereign_chain().set(is_sovereign_chain);
        self.set_paused(true);

        if !is_sovereign_chain {
            match (wegld_identifier, sov_token_prefix) {
                (Some(ticker), Some(prefix)) => {
                    let identifier = TokenIdentifier::from(ticker);
                    if identifier.is_valid_esdt_identifier() {
                        self.wegld_identifier().set(identifier);
                        self.sovereign_tokens_prefix().set(prefix);
                    }
                }
                (Some(_), None) => {
                    sc_panic!("Sovereign Token Prefix must be set in Mainchain")
                }
                (None, Some(_)) => {
                    sc_panic!("WEGLG identifier must be set in Mainchain")
                }
                _ => sc_panic!(
                    "Both WEGLD identifier and Sovereign Token Prefix must be set in Mainchain"
                ),
            }
        }
    }

    #[only_owner]
    #[endpoint(setFeeMarketAddress)]
    fn set_fee_market_address(&self, fee_market_address: ManagedAddress) {
        self.require_sc_address(&fee_market_address);

        self.fee_market_address().set(fee_market_address);
    }

    #[only_owner]
    #[endpoint(setHeaderVerifierAddress)]
    fn set_header_verifier_address(&self, header_verifier_address: ManagedAddress) {
        self.require_sc_address(&header_verifier_address);

        self.header_verifier_address().set(&header_verifier_address);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
