#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod burn_tokens;
pub mod events;
pub mod mint_tokens;
pub mod token_handler_proxy;

#[multiversx_sc::contract]
pub trait TokenHandler {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
