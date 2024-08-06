#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod common_storage;
pub mod token_handler_proxy;
pub mod transfer_tokens;

#[multiversx_sc::contract]
pub trait TokenHandler:
    transfer_tokens::TransferTokensModule + common_storage::CommonStorage
{
    #[init]
    fn init(&self, factory_sc_address: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&factory_sc_address),
            "The caller must be a SC"
        );

        self.factory_sc_address().set(&factory_sc_address);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(whitelistEnshrineEsdt)]
    fn whitelist_enshrine_esdt(&self, enshrine_esdt_address: ManagedAddress<Self::Api>) {
        require!(
            self.blockchain().is_smart_contract(&enshrine_esdt_address),
            "Address passed to be registered is not a valid smart contract address"
        );

        self.enshrine_esdt_whitelist().insert(enshrine_esdt_address);
    }
}
