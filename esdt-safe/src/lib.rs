#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod from_sovereign;
pub mod to_sovereign;
pub mod esdt_safe_proxy;

#[multiversx_sc::contract]
pub trait EsdtSafe:
    to_sovereign::create_tx::CreateTxModule
    + to_sovereign::events::EventsModule
    + to_sovereign::refund::RefundModule
    + to_sovereign::set_tx_status::SetTxStatusModule
    + bls_signature::BlsSignatureModule
    + from_sovereign::events::EventsModule
    + from_sovereign::refund::RefundModule
    + from_sovereign::token_mapping::TokenMappingModule
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

    #[only_owner]
    #[endpoint(setSovereignBridgeAddress)]
    fn set_sovereign_bridge_address(&self, bridge_address: ManagedAddress) {
        self.require_sc_address(&bridge_address);

        self.sovereign_bridge_address().set(bridge_address);
    }

    #[upgrade]
    fn upgrade(&self) {}

}
