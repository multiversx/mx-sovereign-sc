use crate::TransferData;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type BatchId = u64;
pub type TxId = u64;
pub type GasLimit = u64;
pub type TxNonce = u64;

pub type BlockNonce = u64;
pub type SenderAddress<M> = ManagedAddress<M>;
pub type ReceiverAddress<M> = ManagedAddress<M>;
pub type TxAsMultiValue<M> = MultiValue7<
    BlockNonce,
    TxNonce,
    SenderAddress<M>,
    ReceiverAddress<M>,
    ManagedVec<M, EsdtTokenPayment<M>>,
    ManagedVec<M, EsdtTokenData<M>>,
    Option<TransferData<M>>,
>;
pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;
pub type EventPaymentTuple<M> = MultiValue3<TokenIdentifier<M>, u64, EsdtTokenData<M>>;
pub type TxBatchSplitInFields<M> = MultiValue2<BatchId, MultiValueEncoded<M, TxAsMultiValue<M>>>;
pub type ExtractedFeeResult<M> =
    MultiValue2<OptionalValue<EsdtTokenPayment<M>>, ManagedVec<M, EsdtTokenPayment<M>>>;
pub type OptionalValueTransferDataTuple<M> =
    OptionalValue<MultiValue3<GasLimit, ManagedBuffer<M>, ManagedVec<M, ManagedBuffer<M>>>>;
pub type StakeMultiArg<M> = MultiValue2<TokenIdentifier<M>, BigUint<M>>;
