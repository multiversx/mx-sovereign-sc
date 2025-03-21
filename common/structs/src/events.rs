use crate::aliases::EventPaymentTuple;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct EventPayment<M: ManagedTypeApi> {
    pub identifier: TokenIdentifier<M>,
    pub nonce: u64,
    pub data: EsdtTokenData<M>,
}

impl<M: ManagedTypeApi> EventPayment<M> {
    pub fn new(identifier: TokenIdentifier<M>, nonce: u64, data: EsdtTokenData<M>) -> Self {
        EventPayment {
            identifier,
            nonce,
            data,
        }
    }

    pub fn map_to_tuple_multi_value(
        array: MultiValueEncoded<M, Self>,
    ) -> MultiValueEncoded<M, EventPaymentTuple<M>> {
        array.into_iter().map(|payment| payment.into()).collect()
    }
}

impl<M: ManagedTypeApi> From<EventPaymentTuple<M>> for EventPayment<M> {
    fn from(value: EventPaymentTuple<M>) -> Self {
        let (identifier, nonce, data) = value.into_tuple();

        EventPayment::new(identifier, nonce, data)
    }
}

impl<M: ManagedTypeApi> From<EventPayment<M>> for EventPaymentTuple<M> {
    fn from(value: EventPayment<M>) -> EventPaymentTuple<M> {
        MultiValue3((value.identifier, value.nonce, value.data))
    }
}
