#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod burn_tokens;
pub mod common;
pub mod token_handler_proxy;
pub mod transfer_tokens;

#[multiversx_sc::contract]
pub trait TokenHandler:
    transfer_tokens::TransferTokensModule
    + burn_tokens::BurnTokensModule
    + utils::UtilsModule
    + common::storage::CommonStorage
    + tx_batch_module::TxBatchModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
