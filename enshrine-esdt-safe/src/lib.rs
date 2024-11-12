#![no_std]

use multiversx_sc::imports::*;
use transaction::GasLimit;

pub mod common;
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
        token_handler_address: ManagedAddress,
        opt_wegld_identifier: Option<TokenIdentifier>,
        opt_sov_token_prefix: Option<ManagedBuffer>,
    ) {
        self.is_sovereign_chain().set(is_sovereign_chain);
        self.set_paused(true);
        self.token_handler_address().set(token_handler_address);

        if is_sovereign_chain {
            return;
        }

        match opt_wegld_identifier {
            Some(identifier) => {
                require!(
                    identifier.is_valid_esdt_identifier(),
                    "Sent Identifier is not valid"
                );

                self.wegld_identifier().set(identifier);
            }

            None => sc_panic!("WEGLG identifier must be set in Mainchain"),
        }

        match opt_sov_token_prefix {
            Some(prefix) => self.sovereign_tokens_prefix().set(prefix),
            None => sc_panic!("Sovereign Token Prefix must be set in Mainchain"),
        }

        let caller = self.blockchain().get_caller();
        self.initiator_address().set(caller);
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

    #[only_owner]
    #[endpoint(setMaxTxGasLimit)]
    fn set_max_user_tx_gas_limit(&self, max_user_tx_gas_limit: GasLimit) {
        self.max_user_tx_gas_limit().set(max_user_tx_gas_limit);
    }

    #[only_owner]
    #[endpoint(setBannedEndpoint)]
    fn set_banned_endpoint(&self, endpoint_name: ManagedBuffer) {
        self.banned_endpoint_names().insert(endpoint_name);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
