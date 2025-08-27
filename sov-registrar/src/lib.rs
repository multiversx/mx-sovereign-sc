#![no_std]

multiversx_sc::imports!();

pub mod config_operations;
pub mod fee_operations;

#[multiversx_sc::contract]
pub trait SovRegistrar: fee_operations::FeeOperationsModule
// + config_operations::ConfigOperationsModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
