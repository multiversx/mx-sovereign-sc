#![no_std]

use multiversx_sc::imports::*;

pub mod common;
pub mod delegation;
pub mod delegation_proxy;
pub mod liquid_staking_proxy;
pub mod liquidity_pools;

#[multiversx_sc::contract]
pub trait LiquidStaking:
    liquidity_pools::LiquidityPoolModule
    + delegation::DelegationModule
    + common::storage::CommonStorageModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

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

    #[only_owner]
    #[endpoint(registerHeaderVerifierAddress)]
    fn register_header_verifier_address(&self, header_verifier_address: ManagedAddress) {
        self.header_verifier_address().set(header_verifier_address);
    }

    #[endpoint(registerBlsKeys)]
    fn register_bls_keys(&self, bls_keys: MultiValueEncoded<ManagedBuffer>) {
        let caller = self.blockchain().get_caller();
        self.require_caller_header_verifier(&caller);

        self.registered_bls_keys().extend(bls_keys);
    }

    #[endpoint(registerBlsKeys)]
    fn unregister_bls_keys(&self, bls_keys: MultiValueEncoded<ManagedBuffer>) {
        let caller = self.blockchain().get_caller();
        self.require_caller_header_verifier(&caller);

        let mut bls_keys_mapper = self.registered_bls_keys();

        for bls_key in bls_keys {
            bls_keys_mapper.swap_remove(&bls_key);
        }
    }
}
