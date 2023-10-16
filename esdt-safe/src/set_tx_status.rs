use transaction::{transaction_status::TransactionStatus, BatchId};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SetTxStatusModule:
    crate::events::EventsModule
    + crate::refund::RefundModule
    + token_module::TokenModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
{
    /// Sets the statuses for the transactions, after they were executed on the Sovereign side.
    ///
    /// Only TransactionStatus::Executed (3) and TransactionStatus::Rejected (4) values are allowed.
    /// Number of provided statuses must be equal to number of transactions in the batch.
    #[only_owner]
    #[endpoint(setTransactionBatchStatus)]
    fn set_transaction_batch_status(
        &self,
        batch_id: BatchId,
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

        for (tx, tx_status) in tx_batch.iter().zip(tx_statuses.to_vec().iter()) {
            // Since tokens don't exist in the EsdtSafe in the case of a refund transaction
            // we have no tokens to burn, nor to refund
            if tx.is_refund_tx {
                continue;
            }

            match tx_status {
                TransactionStatus::Executed => {
                    // local burn role might be removed while tx is executed
                    // tokens will remain locked forever in that case
                    // otherwise, the whole batch would fail
                    for token in &tx.tokens {
                        if self.is_burn_role_set(&token) {
                            self.send().esdt_local_burn(
                                &token.token_identifier,
                                token.token_nonce,
                                &token.amount,
                            )
                        }
                    }
                }
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
