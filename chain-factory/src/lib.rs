#![no_std]

use multiversx_sc_modules::only_admin;

multiversx_sc::imports!();

pub mod factory;

#[multiversx_sc::contract]
pub trait ChainFactoryContract:
    factory::FactoryModule + utils::UtilsModule + only_admin::OnlyAdminModule
{
    #[init]
    fn init(
        &self,
        sovereign_forge_address: ManagedAddress,
        chain_config_template: ManagedAddress,
        header_verifier_template: ManagedAddress,
        cross_chain_operation_template: ManagedAddress,
        fee_market_template: ManagedAddress,
    ) {
        self.require_sc_address(&sovereign_forge_address);
        self.require_sc_address(&chain_config_template);
        self.require_sc_address(&header_verifier_template);
        self.require_sc_address(&cross_chain_operation_template);
        self.require_sc_address(&fee_market_template);

        self.add_admin(sovereign_forge_address);
        self.chain_config_template().set(chain_config_template);
        self.header_verifier_template()
            .set(header_verifier_template);
        self.enshrine_esdt_safe_template()
            .set(cross_chain_operation_template);
        self.fee_market_template().set(fee_market_template);
    }

    // TODO: Has to be voted first
    #[upgrade]
    fn upgrade(&self) {}
}
