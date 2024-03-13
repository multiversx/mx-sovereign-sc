#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod transaction_status;

// revert protection
pub const MIN_BLOCKS_FOR_FINALITY: u64 = 10;

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
    ManagedVec<M, StolenFromFrameworkEsdtTokenData<M>>,
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

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct OperationData<M: ManagedTypeApi> {
    pub op_nonce: TxId,
    pub opt_transfer_data: Option<TransferData<M>>,
}

// Temporary until Clone is implemented for EsdtTokenData
#[derive(
    TopDecode, TopEncode, NestedDecode, NestedEncode, TypeAbi, Debug, ManagedVecItem, Clone,
)]
pub struct StolenFromFrameworkEsdtTokenData<M: ManagedTypeApi> {
    pub token_type: EsdtTokenType,
    pub amount: BigUint<M>,
    pub frozen: bool,
    pub hash: ManagedBuffer<M>,
    pub name: ManagedBuffer<M>,
    pub attributes: ManagedBuffer<M>,
    pub creator: ManagedAddress<M>,
    pub royalties: BigUint<M>,
    pub uris: ManagedVec<M, ManagedBuffer<M>>,
}

impl<M: ManagedTypeApi> Default for StolenFromFrameworkEsdtTokenData<M> {
    fn default() -> Self {
        StolenFromFrameworkEsdtTokenData {
            token_type: EsdtTokenType::Fungible,
            amount: BigUint::zero(),
            frozen: false,
            hash: ManagedBuffer::new(),
            name: ManagedBuffer::new(),
            attributes: ManagedBuffer::new(),
            creator: ManagedAddress::zero(),
            royalties: BigUint::zero(),
            uris: ManagedVec::new(),
        }
    }
}

impl<M: ManagedTypeApi> From<EsdtTokenData<M>> for StolenFromFrameworkEsdtTokenData<M> {
    fn from(value: EsdtTokenData<M>) -> Self {
        StolenFromFrameworkEsdtTokenData {
            token_type: value.token_type,
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

impl<M: ManagedTypeApi> Into<EsdtTokenData<M>> for StolenFromFrameworkEsdtTokenData<M> {
    fn into(self) -> EsdtTokenData<M> {
        EsdtTokenData {
            token_type: self.token_type,
            amount: self.amount,
            frozen: self.frozen,
            hash: self.hash,
            name: self.name,
            attributes: self.attributes,
            creator: self.creator,
            royalties: self.royalties,
            uris: self.uris,
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
    pub token_data: ManagedVec<M, StolenFromFrameworkEsdtTokenData<M>>,
    pub opt_transfer_data: Option<TransferData<M>>,
    pub is_refund_tx: bool,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct Operation<M: ManagedTypeApi> {
    pub to: ManagedAddress<M>,
    pub tokens: ManagedVec<M, OperationEsdtPayment<M>>,
    pub data: OperationData<M>,
}

impl<M: ManagedTypeApi> Operation<M> {
    pub fn map_tokens_to_tuple_arr(&self) -> MultiValueEncoded<M, MultiValue3<TokenIdentifier<M>, u64, StolenFromFrameworkEsdtTokenData<M>>> {
        let mut tuple_arr = MultiValueEncoded::new();

        for token in &self.tokens {
            tuple_arr.push(MultiValue3::from((token.token_identifier, token.token_nonce, token.token_data)));
        }

        tuple_arr
    }
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct OperationEsdtPayment<M: ManagedTypeApi> {
    pub token_identifier: TokenIdentifier<M>,
    pub token_nonce: u64,
    pub token_data: StolenFromFrameworkEsdtTokenData<M>,
}

impl<M: ManagedTypeApi> Into<EsdtTokenPayment<M>> for OperationEsdtPayment<M> {
    fn into(self) -> EsdtTokenPayment<M> {
        EsdtTokenPayment {
            token_identifier: self.token_identifier,
            token_nonce: self.token_nonce,
            amount: self.token_data.amount,
        }
    }
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
