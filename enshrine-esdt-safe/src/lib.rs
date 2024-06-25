#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod from_sovereign;
pub mod to_sovereign;
pub mod enshrine_esdt_safe_proxy;

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
{
    #[init]
    fn init(
        &self,
        is_sovereign_chain: bool,
    ) {
        self.is_sovereign_chain().set(is_sovereign_chain);
        self.set_paused(true);
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
    
    #[storage_mapper("isSovereignChain")]
    fn is_sovereign_chain(&self) -> SingleValueMapper<bool>;
}
