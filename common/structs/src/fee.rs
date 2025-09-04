use multiversx_sc::api::CryptoApi;

use crate::{
    aliases::{GasLimit, TxNonce},
    generate_hash::GenerateHash,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, PartialEq)]
pub enum FeeType<M: ManagedTypeApi> {
    None,
    Fixed {
        token: EgldOrEsdtTokenIdentifier<M>,
        per_transfer: BigUint<M>,
        per_gas: BigUint<M>,
    },
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedDecode, Clone)]
pub struct AddUsersToWhitelistOperation<M: ManagedTypeApi> {
    pub users: ManagedVec<M, ManagedAddress<M>>,
    pub nonce: TxNonce,
}

impl<A: CryptoApi> GenerateHash<A> for AddUsersToWhitelistOperation<A> {}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedDecode, Clone)]
pub struct RemoveUsersFromWhitelistOperation<M: ManagedTypeApi> {
    pub users: ManagedVec<M, ManagedAddress<M>>,
    pub nonce: TxNonce,
}

impl<A: CryptoApi> GenerateHash<A> for RemoveUsersFromWhitelistOperation<A> {}

#[type_abi]
#[derive(TopDecode, TopEncode, NestedEncode, NestedDecode, Clone)]
pub struct RemoveFeeOperation<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub nonce: TxNonce,
}

impl<A: CryptoApi> GenerateHash<A> for RemoveFeeOperation<A> {}

#[type_abi]
#[derive(TopDecode, TopEncode, NestedEncode, NestedDecode, Clone)]
pub struct SetFeeOperation<M: ManagedTypeApi> {
    pub fee_struct: FeeStruct<M>,
    pub nonce: TxNonce,
}

impl<A: CryptoApi> GenerateHash<A> for SetFeeOperation<A> {}

#[type_abi]
#[derive(TopDecode, TopEncode, NestedEncode, NestedDecode, Clone)]
pub struct FeeStruct<M: ManagedTypeApi> {
    pub base_token: EgldOrEsdtTokenIdentifier<M>,
    pub fee_type: FeeType<M>,
}

impl<A: CryptoApi> GenerateHash<A> for FeeStruct<A> {}
impl<A: CryptoApi> GenerateHash<A> for EgldOrEsdtTokenIdentifier<A> {}

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct FinalPayment<M: ManagedTypeApi> {
    pub fee: EsdtTokenPayment<M>,
    pub remaining_tokens: EsdtTokenPayment<M>,
}

#[type_abi]
#[derive(TopDecode, TopEncode, NestedEncode, NestedDecode, Clone)]
pub struct DistributeFeesOperation<M: ManagedTypeApi> {
    pub pairs: ManagedVec<M, AddressPercentagePair<M>>,
    pub nonce: TxNonce,
}

impl<A: CryptoApi> GenerateHash<A> for DistributeFeesOperation<A> {}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem)]
pub struct AddressPercentagePair<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub percentage: usize,
}

impl<A: CryptoApi> GenerateHash<A> for AddressPercentagePair<A> {}

pub struct SubtractPaymentArguments<M: ManagedTypeApi> {
    pub fee_token: EgldOrEsdtTokenIdentifier<M>,
    pub per_transfer: BigUint<M>,
    pub per_gas: BigUint<M>,
    pub payment: EsdtTokenPayment<M>,
    pub total_transfers: usize,
    pub opt_gas_limit: OptionalValue<GasLimit>,
}
