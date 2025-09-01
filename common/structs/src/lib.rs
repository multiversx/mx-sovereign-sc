#![no_std]

use multiversx_sc::api::CryptoApi;
use crate::generate_hash::GenerateHash;

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

pub const COMPLETE_SETUP_PHASE_GAS: u64 = 80_000_000;

pub const BLS_KEY_BYTE_LENGTH: usize = 96;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct EsdtInfo<M: ManagedTypeApi> {
    pub token_identifier: TokenIdentifier<M>,
    pub token_nonce: u64,
}

pub struct IssueEsdtArgs<M: ManagedTypeApi> {
    pub sov_token_id: TokenIdentifier<M>,
    pub token_type: EsdtTokenType,
    pub issue_cost: BigUint<M>,
    pub token_display_name: ManagedBuffer<M>,
    pub token_ticker: ManagedBuffer<M>,
    pub num_decimals: usize,
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
