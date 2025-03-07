#![no_std]

multiversx_sc::imports!();

pub mod factory;
pub mod slash;

#[multiversx_sc::contract]
pub trait ChainFactoryContract:
    factory::FactoryModule + slash::SlashModule + utils::UtilsModule
{
    #[init]
    fn init(
        &self,
        validators_contract_address: ManagedAddress,
        chain_config_template: ManagedAddress,
        deploy_cost: BigUint,
    ) {
        self.require_sc_address(&validators_contract_address);
        self.require_sc_address(&chain_config_template);

        self.validators_contract_address()
            .set(validators_contract_address);
        self.chain_config_template().set(chain_config_template);
        self.deploy_cost().set(deploy_cost);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
