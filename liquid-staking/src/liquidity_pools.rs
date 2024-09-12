use multiversx_sc::imports::*;

#[multiversx_sc::module]
pub trait LiquidityPoolModule {
    #[view(delegationAddress)]
    #[storage_mapper("delegationAddress")]
    fn delegation_address(&self) -> SingleValueMapper<ManagedAddress>;
}
