#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;
use operation::SovereignConfig;

pub mod deposit;

#[multiversx_sc::contract]
pub trait FromSovereign: deposit::DepositModule + cross_chain::CrossChainCommon {
    #[init]
    fn init(&self, sovereign_config: SovereignConfig<Self::Api>) {
        self.sovereign_config().set(sovereign_config);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
