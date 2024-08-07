#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod burn_tokens;
pub mod common;
pub mod events;
pub mod mint_tokens;
pub mod token_handler_proxy;

#[multiversx_sc::contract]
pub trait TokenHandler:
    mint_tokens::MintTokens
    + burn_tokens::BurnTokens
    + utils::UtilsModule
    + common::storage::CommonStorage
{
    #[init]
    fn init(&self, chain_prefix: ManagedBuffer) {
        self.sov_prefix().set(chain_prefix);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
