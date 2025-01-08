#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;
use multiversx_sc_modules::only_admin;

pub mod common_storage;
pub mod transfer_tokens;

#[multiversx_sc::contract]
pub trait TokenHandler:
    transfer_tokens::TransferTokensModule + common_storage::CommonStorage + only_admin::OnlyAdminModule
{
    #[init]
    fn init(&self, chain_factory_master: ManagedAddress) {
        self.blockchain().is_smart_contract(&chain_factory_master);

        self.add_admin(chain_factory_master);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_admin]
    #[endpoint(whitelistEnshrineEsdt)]
    fn whitelist_enshrine_esdt(&self, enshrine_esdt_address: ManagedAddress<Self::Api>) {
        require!(
            self.blockchain().is_smart_contract(&enshrine_esdt_address),
            "Address passed to be registered is not a valid smart contract address"
        );

        self.enshrine_esdt_whitelist().insert(enshrine_esdt_address);
    }
}
