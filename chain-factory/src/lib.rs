#![no_std]

use multiversx_sc_modules::only_admin;

multiversx_sc::imports!();

pub mod chain_factory_proxy;
pub mod common;
pub mod factory;
pub mod slash;

#[multiversx_sc::contract]
pub trait ChainFactoryContract:
    factory::FactoryModule
    + slash::SlashModule
    + utils::UtilsModule
    + bls_signature::BlsSignatureModule
    + only_admin::OnlyAdminModule
    + crate::common::storage::CommonStorage
    + crate::common::utils::UtilsModule
{
    #[init]
    fn init(
        &self,
        validators_contract_address: ManagedAddress,
        chain_config_template: ManagedAddress,
        header_verifier_template: ManagedAddress,
        cross_chain_operation_template: ManagedAddress,
        fee_market_template: ManagedAddress,
        token_handler_template: ManagedAddress,
        deploy_cost: BigUint,
    ) {
        self.require_sc_address(&validators_contract_address);
        self.require_sc_address(&chain_config_template);
        self.require_sc_address(&header_verifier_template);
        self.require_sc_address(&cross_chain_operation_template);
        self.require_sc_address(&fee_market_template);
        self.require_sc_address(&token_handler_template);

        self.validators_contract_address()
            .set(validators_contract_address);
        self.chain_config_template().set(chain_config_template);
        self.header_verifier_template()
            .set(header_verifier_template);
        self.cross_chain_operations_template()
            .set(cross_chain_operation_template);
        self.fee_market_template().set(fee_market_template);
        self.token_handler_template().set(token_handler_template);
        self.deploy_cost().set(deploy_cost);
    }

    // TODO: Has to be voted first
    #[upgrade]
    fn upgrade(&self) {}
}
