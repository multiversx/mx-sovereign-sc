use transaction::{
    transaction_status::TransactionStatus, BatchId, GasLimit, PaymentsVec, TransferData, TxId,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct DepositEvent<M: ManagedTypeApi> {
    pub tx_nonce: TxId,
    pub opt_function: Option<ManagedBuffer<M>>,
    pub opt_arguments: Option<ManagedVec<M, ManagedBuffer<M>>>,
    pub opt_gas_limit: Option<GasLimit>,
}

impl<M: ManagedTypeApi> DepositEvent<M> {
    pub fn from(tx_nonce: TxId, opt_transfer_data: &OptionalValue<TransferData<M>>) -> Self {
        match opt_transfer_data {
            OptionalValue::Some(transfer_data) => DepositEvent {
                tx_nonce,
                opt_function: Some(transfer_data.function.clone()),
                opt_arguments: Some(transfer_data.args.clone()),
                opt_gas_limit: Some(transfer_data.gas_limit),
            },
            OptionalValue::None => DepositEvent {
                tx_nonce,
                opt_function: None,
                opt_arguments: None,
                opt_gas_limit: None,
            },
        }
    }
}

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("deposit")]
    fn deposit_event(
        &self,
        #[indexed] dest_address: &ManagedAddress,
        #[indexed] tokens: &PaymentsVec<Self::Api>,
        event_data: DepositEvent<Self::Api>,
    );

    #[event("setStatusEvent")]
    fn set_status_event(
        &self,
        #[indexed] batch_id: BatchId,
        #[indexed] tx_id: TxId,
        #[indexed] tx_status: TransactionStatus,
    );
}
