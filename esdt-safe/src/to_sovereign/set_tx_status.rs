use bls_signature::BlsSignature;
use transaction::{transaction_status::TransactionStatus, BatchId};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SetTxStatusModule:
    bls_signature::BlsSignatureModule
    + super::events::EventsModule
    + super::refund::RefundModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
{
    /// Sets the statuses for the transactions, after they were executed on the Sovereign side.
    ///
    /// Only TransactionStatus::Executed (3) and TransactionStatus::Rejected (4) values are allowed.
    /// Number of provided statuses must be equal to number of transactions in the batch.
    #[endpoint(setTransactionBatchStatus)]
    fn set_transaction_batch_status(
        &self,
        batch_id: BatchId,
        signature: BlsSignature<Self::Api>,
        tx_statuses: MultiValueEncoded<TransactionStatus>,
    ) {
        let first_batch_id = self.first_batch_id().get();
        require!(
            batch_id == first_batch_id,
            "Batches must be processed in order"
        );

        let mut tx_batch = self.pending_batches(batch_id);
        require!(
            tx_batch.len() == tx_statuses.len(),
            "Invalid number of statuses provided"
        );

        let mut serialized_data = ManagedBuffer::new();
        let tx_statuses_vec = tx_statuses.to_vec();

        let _ = batch_id.dep_encode(&mut serialized_data);
        for status in &tx_statuses_vec {
            let _ = status.dep_encode(&mut serialized_data);
        }

        self.multi_verify_signature(&serialized_data, &signature);

        for (tx, tx_status) in tx_batch.iter().zip(tx_statuses_vec.iter()) {
            // Since tokens don't exist in the EsdtSafe in the case of a refund transaction
            // we have no tokens to burn, nor to refund
            if tx.is_refund_tx {
                continue;
            }

            match tx_status {
                TransactionStatus::Executed => {}
                TransactionStatus::Rejected => {
                    for token in &tx.tokens {
                        self.mark_refund(&tx.from, &token);
                    }
                }
                _ => {
                    sc_panic!("Transaction status may only be set to Executed or Rejected");
                }
            }

            self.set_status_event(batch_id, tx.nonce, tx_status);
        }

        self.clear_first_batch(&mut tx_batch);
    }
}
