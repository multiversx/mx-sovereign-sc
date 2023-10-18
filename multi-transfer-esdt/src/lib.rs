#![no_std]

multiversx_sc::imports!();

use tx_batch_module::FIRST_BATCH_ID;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = u64::MAX;

pub mod bls_signature;
pub mod events;
pub mod refund;
pub mod token_mapping;
pub mod transfer_tokens;

#[multiversx_sc::contract]
pub trait MultiTransferEsdt:
    bls_signature::BlsSignatureModule
    + events::EventsModule
    + refund::RefundModule
    + token_mapping::TokenMappingModule
    + transfer_tokens::TransferTokensModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    /// Needs to be Payable by SC to receive the tokens from EsdtSafe
    #[init]
    fn init(&self, min_valid_signers: u32, signers: MultiValueEncoded<ManagedAddress>) {
        self.max_tx_batch_size().set(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set(FIRST_BATCH_ID);
        self.last_batch_id().set(FIRST_BATCH_ID);

        self.set_min_valid_signers(min_valid_signers);
        self.add_signers(signers);
    }

    #[endpoint]
    fn upgrade(&self) {}
}
