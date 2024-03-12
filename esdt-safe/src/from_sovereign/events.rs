use transaction::{BatchId, TxId};

use crate::to_sovereign::events::DepositEvent;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("addRefundTransactionEvent")]
    fn add_refund_transaction_event(&self, #[indexed] tx_id: TxId, #[indexed] original_tx_id: TxId);

    #[event("transferPerformedEvent")]
    fn transfer_performed_event(
        &self,
        #[indexed] hash_of_hashes: ManagedBuffer,
        #[indexed] hash_of_bridge_op: ManagedBuffer,
    );

    #[event("transferFailedInvalidToken")]
    fn transfer_failed_invalid_token(&self, #[indexed] batch_id: BatchId, #[indexed] tx_id: TxId);

    #[event("transferFailedFrozenDestinationAccount")]
    fn transfer_failed_frozen_destination_account(
        &self,
        #[indexed] batch_id: BatchId,
        #[indexed] tx_id: TxId,
    );

    #[event("transferOverMaxAmount")]
    fn transfer_over_max_amount(&self, #[indexed] batch_id: BatchId, #[indexed] tx_id: TxId);

    #[event("transferFailedExecutionFailed")]
    fn transfer_failed_execution_failed(
        &self,
        #[indexed] dest_address: &ManagedAddress,
        #[indexed] tokens: &MultiValueEncoded<MultiValue3<TokenIdentifier, u64, BigUint>>,
        event_data: DepositEvent<Self::Api>,
    );
}
