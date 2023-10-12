#![no_std]

multiversx_sc::imports!();

use tx_batch_module::FIRST_BATCH_ID;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = u64::MAX;

pub mod events;
pub mod refund;
pub mod transfer_tokens;

#[multiversx_sc::contract]
pub trait MultiTransferEsdt:
    events::EventsModule
    + refund::RefundModule
    + transfer_tokens::TransferTokensModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
{
    #[init]
    fn init(&self) {
        self.max_tx_batch_size().set(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set(FIRST_BATCH_ID);
        self.last_batch_id().set(FIRST_BATCH_ID);
    }

    #[endpoint]
    fn upgrade(&self) {}

    // private

    // events
}
