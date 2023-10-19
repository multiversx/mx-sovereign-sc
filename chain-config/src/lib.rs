#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait ChainConfigContract: utils::UtilsModule {
    #[init]
    fn init(&self) {}

    #[endpoint]
    fn upgrade(&self) {}
}
