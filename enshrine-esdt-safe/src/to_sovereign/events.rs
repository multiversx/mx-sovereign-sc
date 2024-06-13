use transaction::{
    transaction_status::TransactionStatus, BatchId, OperationData, TxId
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("deposit")]
    fn deposit_event(
        // TODO: Use ManagedVec of EsdtTokenPaymentInfo(EsdtTokenDataPayment, EsdtTokenData)
        &self,
        #[indexed] dest_address: &ManagedAddress,
        #[indexed] tokens: &MultiValueEncoded<MultiValue3<TokenIdentifier, u64, EsdtTokenData>>,
        event_data: OperationData<Self::Api>,
    );

    #[event("setStatusEvent")]
    fn set_status_event(
        &self,
        #[indexed] batch_id: BatchId,
        #[indexed] tx_id: TxId,
        #[indexed] tx_status: TransactionStatus,
    );
}
