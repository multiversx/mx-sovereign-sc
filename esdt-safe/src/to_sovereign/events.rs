use transaction::{transaction_status::TransactionStatus, BatchId, TxId};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("createTransactionEvent")]
    fn create_transaction_event(&self, #[indexed] batch_id: BatchId, #[indexed] tx_id: TxId);

    #[event("setStatusEvent")]
    fn set_status_event(
        &self,
        #[indexed] batch_id: BatchId,
        #[indexed] tx_id: TxId,
        #[indexed] tx_status: TransactionStatus,
    );
}
