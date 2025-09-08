use aliases::{GasLimit, OptionalValueTransferDataTuple, TxId};
use multiversx_sc::api::CryptoApi;

use crate::{
    aliases::{self, EventPaymentTuple, TransferDataTuple},
    events::EventPayment,
    generate_hash::GenerateHash,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct Operation<M: ManagedTypeApi> {
    pub to: ManagedAddress<M>,
    pub tokens: ManagedVec<M, OperationEsdtPayment<M>>,
    pub data: OperationData<M>,
}

impl<A: CryptoApi> GenerateHash<A> for Operation<A> {}

impl<M: ManagedTypeApi> Operation<M> {
    #[inline]
    pub fn new(
        to: ManagedAddress<M>,
        tokens: ManagedVec<M, OperationEsdtPayment<M>>,
        data: OperationData<M>,
    ) -> Self {
        Operation { to, tokens, data }
    }

    pub fn map_tokens_to_multi_value_encoded(&self) -> MultiValueEncoded<M, EventPaymentTuple<M>> {
        let mut tuples = MultiValueEncoded::new();

        for token in &self.tokens {
            let event_payment: EventPaymentTuple<M> = EventPaymentTuple::from(EventPayment {
                identifier: token.token_identifier.clone(),
                nonce: token.token_nonce,
                data: token.token_data.clone(),
            });
            tuples.push(event_payment);
        }

        tuples
    }
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct TransferData<M: ManagedTypeApi> {
    pub gas_limit: GasLimit,
    pub function: ManagedBuffer<M>,
    pub args: ManagedVec<M, ManagedBuffer<M>>,
}

impl<M: ManagedTypeApi> TransferData<M> {
    #[inline]
    pub fn new(
        gas_limit: GasLimit,
        function: ManagedBuffer<M>,
        args: ManagedVec<M, ManagedBuffer<M>>,
    ) -> Self {
        TransferData {
            gas_limit,
            function,
            args,
        }
    }

    pub fn from_optional_value(
        opt_value_transfer_data: OptionalValueTransferDataTuple<M>,
    ) -> Option<Self> {
        match opt_value_transfer_data {
            OptionalValue::Some(multi_value_transfer_data) => {
                Option::Some(multi_value_transfer_data.into())
            }
            OptionalValue::None => Option::None,
        }
    }
}

impl<M: ManagedTypeApi> From<TransferDataTuple<M>> for TransferData<M> {
    fn from(value: TransferDataTuple<M>) -> Self {
        let (gas_limit, function, vec) = value.into_tuple();
        TransferData::new(gas_limit, function, vec.to_vec())
    }
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct OperationData<M: ManagedTypeApi> {
    pub op_nonce: TxId,
    pub op_sender: ManagedAddress<M>,
    pub opt_transfer_data: Option<TransferData<M>>,
}

impl<M: ManagedTypeApi> OperationData<M> {
    #[inline]
    pub fn new(
        op_nonce: TxId,
        op_sender: ManagedAddress<M>,
        opt_transfer_data: Option<TransferData<M>>,
    ) -> Self {
        OperationData {
            op_nonce,
            op_sender,
            opt_transfer_data,
        }
    }
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct OperationTuple<M: ManagedTypeApi> {
    pub op_hash: ManagedBuffer<M>,
    pub operation: Operation<M>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct OperationEsdtPayment<M: ManagedTypeApi> {
    pub token_identifier: EgldOrEsdtTokenIdentifier<M>,
    pub token_nonce: u64,
    pub token_data: EsdtTokenData<M>,
}

impl<M: ManagedTypeApi> OperationEsdtPayment<M> {
    #[inline]
    pub fn new(
        token_identifier: EgldOrEsdtTokenIdentifier<M>,
        token_nonce: u64,
        token_data: EsdtTokenData<M>,
    ) -> Self {
        Self {
            token_identifier,
            token_nonce,
            token_data,
        }
    }
}

impl<M: ManagedTypeApi> From<OperationEsdtPayment<M>> for EgldOrEsdtTokenPayment<M> {
    #[inline]
    fn from(payment: OperationEsdtPayment<M>) -> Self {
        EgldOrEsdtTokenPayment {
            token_identifier: payment.token_identifier,
            token_nonce: payment.token_nonce,
            amount: payment.token_data.amount,
        }
    }
}

impl<M: ManagedTypeApi> Default for OperationEsdtPayment<M> {
    fn default() -> Self {
        OperationEsdtPayment {
            token_identifier: EgldOrEsdtTokenIdentifier::from(ManagedBuffer::new()),
            token_nonce: 0,
            token_data: EsdtTokenData::default(),
        }
    }
}

impl<M: ManagedTypeApi> OperationTuple<M> {
    #[inline]
    pub fn new(op_hash: ManagedBuffer<M>, operation: Operation<M>) -> Self {
        OperationTuple { op_hash, operation }
    }
}
