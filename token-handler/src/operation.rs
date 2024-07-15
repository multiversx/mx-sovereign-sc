multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type TxId = u64;
pub type GasLimit = u64;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct Operation<M: ManagedTypeApi> {
    pub to: ManagedAddress<M>,
    pub tokens: ManagedVec<M, OperationEsdtPayment<M>>,
    pub data: OperationData<M>,
}

impl<M: ManagedTypeApi> Operation<M> {
    pub fn get_tokens_as_tuple_arr(
        &self,
    ) -> MultiValueEncoded<M, MultiValue3<TokenIdentifier<M>, u64, EsdtTokenData<M>>> {
        let mut tuple_arr = MultiValueEncoded::new();

        for token in &self.tokens {
            tuple_arr.push(MultiValue3::from((
                token.token_identifier,
                token.token_nonce,
                token.token_data.into(),
            )));
        }

        tuple_arr
    }
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct OperationData<M: ManagedTypeApi> {
    pub op_nonce: TxId,
    pub op_sender: ManagedAddress<M>,
    pub opt_transfer_data: Option<TransferData<M>>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct OperationTuple<M: ManagedTypeApi> {
    pub op_hash: ManagedBuffer<M>,
    pub operation: Operation<M>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct OperationEsdtPayment<M: ManagedTypeApi> {
    pub token_identifier: TokenIdentifier<M>,
    pub token_nonce: u64,
    pub token_data: StolenFromFrameworkEsdtTokenData<M>,
}

impl<M: ManagedTypeApi> From<OperationEsdtPayment<M>> for EsdtTokenPayment<M> {
    fn from(payment: OperationEsdtPayment<M>) -> Self {
        EsdtTokenPayment {
            token_identifier: payment.token_identifier,
            token_nonce: payment.token_nonce,
            amount: payment.token_data.amount,
        }
    }
}

impl<M: ManagedTypeApi> Default for OperationEsdtPayment<M> {
    fn default() -> Self {
        OperationEsdtPayment {
            token_identifier: TokenIdentifier::from(ManagedBuffer::new()),
            token_nonce: 0,
            token_data: StolenFromFrameworkEsdtTokenData::default(),
        }
    }
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct TransferData<M: ManagedTypeApi> {
    pub gas_limit: GasLimit,
    pub function: ManagedBuffer<M>,
    pub args: ManagedVec<M, ManagedBuffer<M>>,
}

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

impl<M: ManagedTypeApi> From<StolenFromFrameworkEsdtTokenData<M>> for EsdtTokenData<M> {
    fn from(token_data: StolenFromFrameworkEsdtTokenData<M>) -> Self {
        EsdtTokenData {
            token_type: token_data.token_type,
            amount: token_data.amount,
            frozen: token_data.frozen,
            hash: token_data.hash,
            name: token_data.name,
            attributes: token_data.attributes,
            creator: token_data.creator,
            royalties: token_data.royalties,
            uris: token_data.uris,
        }
    }
}
