#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod transaction_status;

// revert protection
pub const MIN_BLOCKS_FOR_FINALITY: u64 = 10;
pub const TX_MULTIRESULT_NR_FIELDS: usize = 6;

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
    ManagedVec<M, Option<StolenFromFrameworkEsdtTokenData<M>>>,
    Option<TransferData<M>>,
>;
pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;
pub type TxBatchSplitInFields<M> = MultiValue2<BatchId, MultiValueEncoded<M, TxAsMultiValue<M>>>;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct TransferData<M: ManagedTypeApi> {
    pub gas_limit: GasLimit,
    pub function: ManagedBuffer<M>,
    pub args: ManagedVec<M, ManagedBuffer<M>>,
}

#[derive(
    TopDecode,
    TopEncode,
    NestedDecode,
    NestedEncode,
    Clone,
    PartialEq,
    Eq,
    Debug,
    TypeAbi,
    ManagedVecItem,
)]
pub enum StolenFromFrameworkEsdtTokenType {
    Fungible,
    NonFungible,
    SemiFungible,
    Meta,
    Invalid,
}

impl From<EsdtTokenType> for StolenFromFrameworkEsdtTokenType {
    fn from(value: EsdtTokenType) -> Self {
        match value {
            EsdtTokenType::Fungible => StolenFromFrameworkEsdtTokenType::Fungible,
            EsdtTokenType::NonFungible => StolenFromFrameworkEsdtTokenType::NonFungible,
            EsdtTokenType::SemiFungible => StolenFromFrameworkEsdtTokenType::SemiFungible,
            EsdtTokenType::Meta => StolenFromFrameworkEsdtTokenType::Meta,
            EsdtTokenType::Invalid => StolenFromFrameworkEsdtTokenType::Invalid,
        }
    }
}

#[derive(
    TopDecode, TopEncode, NestedDecode, NestedEncode, TypeAbi, Debug, ManagedVecItem, Clone,
)]
pub struct StolenFromFrameworkEsdtTokenData<M: ManagedTypeApi> {
    pub token_type: StolenFromFrameworkEsdtTokenType,
    pub amount: BigUint<M>,
    pub frozen: bool,
    pub hash: ManagedBuffer<M>,
    pub name: ManagedBuffer<M>,
    pub attributes: ManagedBuffer<M>,
    pub creator: ManagedAddress<M>,
    pub royalties: BigUint<M>,
    pub uris: ManagedVec<M, ManagedBuffer<M>>,
}

impl<M: ManagedTypeApi> From<EsdtTokenData<M>> for StolenFromFrameworkEsdtTokenData<M> {
    fn from(value: EsdtTokenData<M>) -> Self {
        StolenFromFrameworkEsdtTokenData {
            token_type: value.token_type.into(),
            amount: value.amount,
            frozen: value.frozen,
            hash: value.hash,
            name: value.name,
            attributes: value.attributes,
            creator: value.creator,
            royalties: value.royalties,
            uris: value.uris,
        }
    }
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct Transaction<M: ManagedTypeApi> {
    pub block_nonce: BlockNonce,
    pub nonce: TxNonce,
    pub from: ManagedAddress<M>,
    pub to: ManagedAddress<M>,
    pub tokens: ManagedVec<M, EsdtTokenPayment<M>>,
    pub token_data: ManagedVec<M, Option<StolenFromFrameworkEsdtTokenData<M>>>,
    pub opt_transfer_data: Option<TransferData<M>>,
    pub is_refund_tx: bool,
}

impl<M: ManagedTypeApi> From<TxAsMultiValue<M>> for Transaction<M> {
    fn from(tx_as_multiresult: TxAsMultiValue<M>) -> Self {
        let (block_nonce, nonce, from, to, tokens, token_data, opt_transfer_data) =
            tx_as_multiresult.into_tuple();

        Transaction {
            block_nonce,
            nonce,
            from,
            to,
            tokens,
            token_data,
            opt_transfer_data,
            is_refund_tx: false,
        }
    }
}

impl<M: ManagedTypeApi> Transaction<M> {
    pub fn into_multiresult(self) -> TxAsMultiValue<M> {
        (
            self.block_nonce,
            self.nonce,
            self.from,
            self.to,
            self.tokens,
            self.token_data,
            self.opt_transfer_data,
        )
            .into()
    }
}
