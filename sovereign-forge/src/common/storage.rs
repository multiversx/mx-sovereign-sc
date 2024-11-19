use multiversx_sc::imports::{SingleValueMapper, UnorderedSetMapper};

use super::utils::ChainContractsMap;

#[multiversx_sc::module]
pub trait StorageModule {
    #[storage_mapper("sovereignsMapper")]
    fn sovereigns_mapper(
        &self,
        sovereign_creator: &ManagedAddress,
    ) -> SingleValueMapper<ChainContractsMap<Self::Api>>;

    #[view(getDeployCost)]
    #[storage_mapper("deployCost")]
    fn deploy_cost(&self) -> SingleValueMapper<BigUint>;

    #[view(getAllChainIds)]
    #[storage_mapper("allChainIds")]
    fn chain_ids(&self) -> UnorderedSetMapper<ManagedBuffer>;
}
