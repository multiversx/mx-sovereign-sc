#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod burn_tokens;
pub mod common;
pub mod events;
pub mod mint_tokens;
pub mod token_handler_proxy;

#[multiversx_sc::contract]
pub trait TokenHandler:
    mint_tokens::MintTokens
    + burn_tokens::BurnTokens
    + utils::UtilsModule
    + common::storage::CommonStorage
{
    #[init]
    fn init(&self, chain_prefix: ManagedBuffer) {
        self.sov_prefix().set(chain_prefix);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(setEnshrineEsdtWhitelist)]
    fn set_enshrine_esdt_whitelist(
        &self,
        enshrine_esdt_addresses: MultiValueEncoded<ManagedAddress<Self::Api>>,
    ) {
        require!(
            !enshrine_esdt_addresses.is_empty(),
            "There are no addresses sent to be registered"
        );

        for esdt_address in enshrine_esdt_addresses.into_iter() {
            require!(
                self.blockchain().is_smart_contract(&esdt_address),
                "One of the addresses passed is not a valid smart contract address"
            );

            self.enshrine_esdt_whitelist().insert(esdt_address);
        }
    }
}
