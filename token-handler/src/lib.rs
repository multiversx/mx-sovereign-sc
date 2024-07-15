#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod burn_tokens;
pub mod mint_tokens;

#[multiversx_sc::contract]
pub trait TokenHandler {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
