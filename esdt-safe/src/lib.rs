#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use tx_batch_module::FIRST_BATCH_ID;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = 100; // ~10 minutes

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
    + token_module::TokenModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::pause::PauseModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(&self, min_valid_signers: u32, signers: MultiValueEncoded<ManagedAddress>) {
        self.max_tx_batch_size().set(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set(FIRST_BATCH_ID);
        self.last_batch_id().set(FIRST_BATCH_ID);
        self.next_batch_id().set(FIRST_BATCH_ID);

        self.set_min_valid_signers(min_valid_signers);
        self.add_signers(signers);

        self.set_paused(true);
    }

    #[endpoint]
    fn upgrade(&self) {}
}
