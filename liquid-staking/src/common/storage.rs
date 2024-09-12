use multiversx_sc::imports::*;
pub type Epoch = u64;

#[multiversx_sc::module]
pub trait CommonStorageModule {
    #[view(getDelegationAddress)]
    #[storage_mapper("delegationAddress")]
    fn delegation_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getDelegatedValue)]
    #[storage_mapper("delegatedValue")]
    fn delegated_value(&self, validator: ManagedAddress) -> SingleValueMapper<BigUint<Self::Api>>;

    #[view(unDelegateEpoch)]
    #[storage_mapper("unDelegateEpoch")]
    fn undelegate_epoch(&self, address: &ManagedAddress) -> SingleValueMapper<Epoch>;

    #[view(getTotalEgldSupply)]
    #[storage_mapper("totalEgldSupply")]
    fn egld_token_supply(&self) -> SingleValueMapper<BigUint>;
}
