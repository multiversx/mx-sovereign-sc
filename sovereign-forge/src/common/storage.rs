use multiversx_sc::imports::{SingleValueMapper, UnorderedSetMapper};

use super::utils::ChainContractsMap;

#[multiversx_sc::module]
pub trait StorageModule {
    #[storage_mapper("sovereignsMapper")]
    fn sovereigns_mapper(
        &self,
        sovereign_creator: &ManagedAddress,
    ) -> SingleValueMapper<ChainContractsMap<Self::Api>>;

    #[storage_mapper("chainFactories")]
    fn chain_factories(&self, shard_id: u32) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("tokenHadlersFactories")]
    fn token_handlers(&self, shard_id: u32) -> SingleValueMapper<ManagedAddress>;

    #[view(getDeployCost)]
    #[storage_mapper("deployCost")]
    fn deploy_cost(&self) -> SingleValueMapper<BigUint>;

    #[view(getAllChainIds)]
    #[storage_mapper("allChainIds")]
    fn chain_ids(&self) -> UnorderedSetMapper<ManagedBuffer>;
}