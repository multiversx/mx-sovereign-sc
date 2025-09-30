#![no_std]

use crate::{generate_hash::GenerateHash, operation::OperationData};
use multiversx_sc::api::CryptoApi;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod aliases;
pub mod configs;
pub mod events;
pub mod fee;
pub mod forge;
pub mod generate_hash;
pub mod operation;

pub const MIN_BLOCKS_FOR_FINALITY: u64 = 10;
pub const DEFAULT_MAX_TX_GAS_LIMIT: u64 = 300_000_000;

pub const PHASE_ONE_ASYNC_CALL_GAS: u64 = 7_500_000;
pub const PHASE_ONE_CALLBACK_GAS: u64 = 3_000_000;

pub const PHASE_TWO_ASYNC_CALL_GAS: u64 = 17_000_000;
pub const PHASE_TWO_CALLBACK_GAS: u64 = 2_000_000;

pub const PHASE_THREE_ASYNC_CALL_GAS: u64 = 16_000_000;
pub const PHASE_THREE_CALLBACK_GAS: u64 = 2_000_000;

pub const PHASE_FOUR_ASYNC_CALL_GAS: u64 = 7_500_000;
pub const PHASE_FOUR_CALLBACK_GAS: u64 = 3_000_000;

pub const COMPLETE_SETUP_PHASE_GAS: u64 = 50_000_000;
pub const COMPLETE_SETUP_PHASE_CALLBACK_GAS: u64 = 3_000_000;

pub const BLS_KEY_BYTE_LENGTH: usize = 96;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct EsdtInfo<M: ManagedTypeApi> {
    pub token_identifier: EgldOrEsdtTokenIdentifier<M>,
    pub token_nonce: u64,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode)]
pub struct ValidatorInfo<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub bls_key: ManagedBuffer<M>,
    pub egld_stake: BigUint<M>,
    pub token_stake: Option<ManagedVec<M, EsdtTokenPayment<M>>>,
}

impl<A: CryptoApi> GenerateHash<A> for ValidatorData<A> {}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode)]
pub struct ValidatorData<M: ManagedTypeApi> {
    pub id: BigUint<M>,
    pub address: ManagedAddress<M>,
    pub bls_key: ManagedBuffer<M>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode)]
pub struct RegisterTokenOperation<M: ManagedTypeApi> {
    pub token_id: EgldOrEsdtTokenIdentifier<M>,
    pub token_type: EsdtTokenType,
    pub token_display_name: ManagedBuffer<M>,
    pub token_ticker: ManagedBuffer<M>,
    pub num_decimals: usize,
    pub data: OperationData<M>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode)]
pub struct RegisterTokenStruct<M: ManagedTypeApi> {
    pub token_id: EgldOrEsdtTokenIdentifier<M>,
    pub token_type: EsdtTokenType,
    pub token_display_name: ManagedBuffer<M>,
    pub token_ticker: ManagedBuffer<M>,
    pub num_decimals: usize,
}

impl<A: CryptoApi> GenerateHash<A> for RegisterTokenOperation<A> {}

#[type_abi]
#[derive(TopEncode, TopDecode, PartialEq, Debug)]
pub enum OperationHashStatus {
    NotLocked = 1,
    Locked,
}
