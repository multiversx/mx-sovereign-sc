multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct ContractInfo<M: ManagedTypeApi> {
    pub id: ScArray,
    pub address: ManagedAddress<M>,
}

impl<M: ManagedTypeApi> ContractInfo<M> {
    pub fn new(id: ScArray, address: ManagedAddress<M>) -> Self {
        ContractInfo { id, address }
    }
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, PartialEq)]
pub enum ScArray {
    ChainFactory,
    Controller,
    HeaderVerifier,
    ESDTSafe,
    FeeMarket,
    TokenHandler,
    ChainConfig,
    Slashing,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, PartialEq)]
pub struct NativeToken<M: ManagedTypeApi> {
    pub ticker: ManagedBuffer<M>,
    pub name: ManagedBuffer<M>,
}
