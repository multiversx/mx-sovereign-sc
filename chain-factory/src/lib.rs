#![no_std]

use multiversx_sc_modules::only_admin;

multiversx_sc::imports!();

pub mod complete_phases;
pub mod factory;
pub mod update_configs;

#[multiversx_sc::contract]
pub trait ChainFactoryContract:
    factory::FactoryModule
    + common_utils::CommonUtilsModule
    + only_admin::OnlyAdminModule
    + update_configs::UpdateConfigsModule
    + complete_phases::CompletePhasesModule
    + custom_events::CustomEventsModule
{
    #[init]
    fn init(
        &self,
        sovereign_forge_address: ManagedAddress,
        chain_config_template: ManagedAddress,
        header_verifier_template: ManagedAddress,
        mvx_esdt_safe_template: ManagedAddress,
        fee_market_template: ManagedAddress,
    ) {
        self.require_sc_address(&sovereign_forge_address);
        self.require_sc_address(&chain_config_template);
        self.require_sc_address(&header_verifier_template);
        self.require_sc_address(&mvx_esdt_safe_template);
        self.require_sc_address(&fee_market_template);

        self.add_admin(sovereign_forge_address);
        self.chain_config_template().set(chain_config_template);
        self.header_verifier_template()
            .set(header_verifier_template);
        self.esdt_safe_template().set(mvx_esdt_safe_template);
        self.fee_market_template().set(fee_market_template);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
