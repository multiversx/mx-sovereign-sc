#![no_std]

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
