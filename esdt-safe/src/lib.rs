#![no_std]
#![allow(non_snake_case)]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use transaction::GasLimit;
use tx_batch_module::FIRST_BATCH_ID;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = 100; // ~10 minutes

pub mod create_tx;
pub mod events;
pub mod refund;
pub mod set_tx_status;

#[multiversx_sc::contract]
pub trait EsdtSafe:
    create_tx::CreateTxModule
    + events::EventsModule
    + refund::RefundModule
    + set_tx_status::SetTxStatusModule
    + token_module::TokenModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::pause::PauseModule
{
    /// sovereign_tx_gas_limit - The gas limit that will be used for transactions on the Sovereign side.
    /// In case of SC gas limits, this value is provided by the user
    /// Will be used to compute the fees for the transfer
    #[init]
    fn init(&self, sovereign_tx_gas_limit: GasLimit) {
        self.sovereign_tx_gas_limit().set(sovereign_tx_gas_limit);

        self.max_tx_batch_size().set(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set(FIRST_BATCH_ID);
        self.last_batch_id().set(FIRST_BATCH_ID);

        self.set_paused(true);
    }

    #[endpoint]
    fn upgrade(&self) {}
}
