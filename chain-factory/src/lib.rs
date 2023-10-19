#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait ChainFactoryContract {
    #[init]
    fn init(&self) {}
}
