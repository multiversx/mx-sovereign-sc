use multiversx_sc::imports::UnorderedSetMapper;

#[multiversx_sc::module]
pub trait StorageModule {
    #[view(getAllChainIds)]
    #[storage_mapper("allChainIds")]
    fn chain_ids(&self) -> UnorderedSetMapper<ManagedBuffer>;
}
