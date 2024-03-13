use transaction::{transaction_status::TransactionStatus, BatchId, OperationData, TxId};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

// #[derive(TypeAbi, TopEncode, TopDecode)]
// pub struct DepositEvent<M: ManagedTypeApi> {
//     pub tx_nonce: TxId,
//     pub opt_gas_limit: Option<GasLimit>,
//     pub opt_function: Option<ManagedBuffer<M>>,
//     pub opt_arguments: Option<ManagedVec<M, ManagedBuffer<M>>>,
// }
//
// impl<M: ManagedTypeApi> DepositEvent<M> {
//     pub fn from(tx_nonce: TxId, opt_transfer_data: &OptionalValue<OperationData<M>>) -> Self {
//         match opt_transfer_data {
//             OptionalValue::Some(transfer_data) => {
//                 let tx_nonce = transfer_data.op_nonce;
//                 match &transfer_data.opt_transfer_data {
//                     Some(some_transfer_data) => DepositEvent {
//                         tx_nonce,
//                         opt_gas_limit: Some(some_transfer_data.gas_limit),
//                         opt_function: Some(some_transfer_data.function.clone()),
//                         opt_arguments: Some(some_transfer_data.args.clone()),
//                     },
//                     None => DepositEvent {
//                         tx_nonce,
//                         opt_function: None,
//                         opt_arguments: None,
//                         opt_gas_limit: None,
//                     },
//                 }
//             }
//             OptionalValue::None => DepositEvent {
//                 tx_nonce,
//                 opt_gas_limit: None,
//                 opt_function: None,
//                 opt_arguments: None,
//             },
//         }
//     }
// }

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
