#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

mod phases;
mod storage;

#[multiversx_sc::contract]
pub trait SovereignForge: phases::PhasesModule + storage::StorageModule {
    #[init]
    fn init(&self, deploy_cost: BigUint) {
        require!(deploy_cost > 0, "The deploy cost can't be a 0 value");
        self.deploy_cost().set(deploy_cost);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
