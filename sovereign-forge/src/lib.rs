#![no_std]

use crate::err_msg;
use multiversx_sc::imports::*;

mod common;
mod phases;

#[multiversx_sc::contract]
pub trait SovereignForge:
    phases::PhasesModule + common::storage::StorageModule + common::utils::UtilsModule
{
    #[init]
    fn init(&self, deploy_cost: BigUint) {
        require!(deploy_cost > 0, "The deploy cost can't be a 0 value");
        self.deploy_cost().set(deploy_cost);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
