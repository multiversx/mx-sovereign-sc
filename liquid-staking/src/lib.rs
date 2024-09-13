#![no_std]

use multiversx_sc::imports::*;

pub mod common;
pub mod delegation;
pub mod liquidity_pools;

#[multiversx_sc::contract]
pub trait LiquidStaking:
    liquidity_pools::LiquidityPoolModule
    + delegation::DelegationModule
    + common::storage::CommonStorageModule
{
    #[init]
    fn init(&self) {}

    #[endpoint(registerDelegationContractAddress)]
    fn register_delegation_address(
        &self,
        contract_name: ManagedBuffer,
        delegation_address: ManagedAddress,
    ) {
        require!(
            self.blockchain().is_smart_contract(&delegation_address),
            "Provided address is not a valid Smart Contract address"
        );

        require!(
            self.delegation_addresses(&contract_name).is_empty(),
            "This contract is already registered"
        );

        self.delegation_addresses(&contract_name)
            .set(delegation_address);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
