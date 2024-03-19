#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use transaction::GasLimit;
use tx_batch_module::FIRST_BATCH_ID;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = 100; // ~10 minutes
const DEFAULT_MAX_USER_TX_GAS_LIMIT: GasLimit = 300_000_000;

pub mod from_sovereign;
pub mod to_sovereign;

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
        min_valid_signers: u32,
        initiator_address: ManagedAddress,
        signers: MultiValueEncoded<ManagedAddress>,
    ) {
        self.is_sovereign_chain().set(is_sovereign_chain);
        self.max_tx_batch_size().set(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);
        self.max_user_tx_gas_limit()
            .set(DEFAULT_MAX_USER_TX_GAS_LIMIT);

        // batch ID 0 is considered invalid
        self.first_batch_id().set(FIRST_BATCH_ID);
        self.last_batch_id().set(FIRST_BATCH_ID);
        self.next_batch_id().set(FIRST_BATCH_ID);

        self.set_min_valid_signers(min_valid_signers);
        self.add_signers(signers);

        self.initiator_address().set(initiator_address);

        self.set_paused(true);

        // Currently, false is the same as 0, which is the default value.
        // If this ever changes, uncomment this line.
        // self.setup_phase_complete.set(false);
    }

    #[only_owner]
    #[endpoint(setFeeMarketAddress)]
    fn set_fee_market_address(&self, fee_market_address: ManagedAddress) {
        self.require_sc_address(&fee_market_address);

        self.fee_market_address().set(fee_market_address);
    }

    #[only_owner]
    #[endpoint(setMultisigAddress)]
    fn set_multisig_address(&self, multisig_address: ManagedAddress) {
        self.require_sc_address(&multisig_address);

        self.multisig_address().set(multisig_address);
    }

    #[only_owner]
    #[endpoint(setSovereignBridgeAddress)]
    fn set_sovereign_bridge_address(&self, bridge_address: ManagedAddress) {
        self.require_sc_address(&bridge_address);

        self.sovereign_bridge_address().set(bridge_address);
    }

    #[endpoint]
    fn upgrade(&self) {}

}
