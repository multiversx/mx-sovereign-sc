use multiversx_sc::imports::{SingleValueMapper, UnorderedSetMapper};

#[multiversx_sc::module]
pub trait StorageModule {
    #[view(getDeployCost)]
    #[storage_mapper("deployCost")]
    fn deploy_cost(&self) -> SingleValueMapper<BigUint>;

    #[view(getAllChainIds)]
    #[storage_mapper("allChainIds")]
    fn chain_ids(&self) -> UnorderedSetMapper<ManagedBuffer>;
}
