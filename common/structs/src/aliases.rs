multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type BatchId = u64;
pub type TxId = u64;
pub type GasLimit = u64;
pub type TxNonce = u64;

pub type BlockNonce = u64;
pub type SenderAddress<M> = ManagedAddress<M>;
pub type ReceiverAddress<M> = ManagedAddress<M>;
pub type EventPaymentTuple<M> = MultiValue3<TokenIdentifier<M>, u64, EsdtTokenData<M>>;
pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;
pub type TransferDataTuple<M> =
    MultiValue3<GasLimit, ManagedBuffer<M>, ManagedVec<M, ManagedBuffer<M>>>;
pub type ExtractedFeeResult<M> =
    MultiValue2<OptionalValue<EsdtTokenPayment<M>>, ManagedVec<M, EsdtTokenPayment<M>>>;
pub type OptionalValueTransferDataTuple<M> =
    OptionalValue<MultiValue3<GasLimit, ManagedBuffer<M>, ManagedVec<M, ManagedBuffer<M>>>>;
pub type StakeMultiArg<M> = MultiValue2<TokenIdentifier<M>, BigUint<M>>;
pub type OptionalTransferData<M> =
    OptionalValue<MultiValue3<GasLimit, ManagedBuffer<M>, ManagedVec<M, ManagedBuffer<M>>>>;
