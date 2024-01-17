#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait FeeMarket {
    #[init]
    fn init(&self) {}
}
