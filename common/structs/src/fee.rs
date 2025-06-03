use multiversx_sc::api::CryptoApi;

use crate::{aliases::GasLimit, generate_hash::GenerateHash};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, PartialEq)]
pub enum FeeType<M: ManagedTypeApi> {
    None,
    Fixed {
        token: TokenIdentifier<M>,
        per_transfer: BigUint<M>,
        per_gas: BigUint<M>,
    },
    AnyToken {
        base_fee_token: TokenIdentifier<M>,
        per_transfer: BigUint<M>,
        per_gas: BigUint<M>,
    },
}

#[type_abi]
#[derive(TopDecode, TopEncode, NestedEncode, NestedDecode, Clone)]
pub struct FeeStruct<M: ManagedTypeApi> {
    pub base_token: TokenIdentifier<M>,
    pub fee_type: FeeType<M>,
}

impl<A: CryptoApi> GenerateHash<A> for FeeStruct<A> {}
impl<A: CryptoApi> GenerateHash<A> for TokenIdentifier<A> {}

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct FinalPayment<M: ManagedTypeApi> {
    pub fee: EsdtTokenPayment<M>,
    pub remaining_tokens: EsdtTokenPayment<M>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, ManagedVecItem)]
pub struct AddressPercentagePair<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub percentage: usize,
}

impl<A: CryptoApi> GenerateHash<A> for AddressPercentagePair<A> {}

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct FeeContext<M: ManagedTypeApi> {
    pub original_caller: ManagedAddress<M>,
    pub total_transfers: usize,
    pub opt_gas_limit: Option<GasLimit>,
}

impl<A: CryptoApi> GenerateHash<A> for FeeContext<A> {}

pub struct SubtractPaymentArguments<M: ManagedTypeApi> {
    pub fee_token: TokenIdentifier<M>,
    pub per_transfer: BigUint<M>,
    pub per_gas: BigUint<M>,
    pub payment: EsdtTokenPayment<M>,
    pub total_transfers: usize,
    pub opt_gas_limit: OptionalValue<GasLimit>,
}
