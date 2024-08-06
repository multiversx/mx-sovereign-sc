#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod token_handler_proxy;
pub mod transfer_tokens;

#[multiversx_sc::contract]
pub trait TokenHandler: transfer_tokens::TransferTokensModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
