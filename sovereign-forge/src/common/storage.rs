use multiversx_sc::{
    imports::{SingleValueMapper, UnorderedSetMapper},
    types::ManagedBuffer,
};

use super::utils::ContractInfo;

type ChainId<M> = ManagedBuffer<M>;

#[multiversx_sc::module]
pub trait StorageModule {
    // TODO: This has to be easily modifiable
    #[storage_mapper("sovereignsMapper")]
    fn sovereigns_mapper(
        &self,
        sovereign_creator: &ManagedAddress,
    ) -> SingleValueMapper<ChainId<Self::Api>>;

    #[storage_mapper("sovereignDeployedContracts")]
    fn sovereign_deployed_contracts(
        &self,
        chain_id: &ManagedBuffer,
    ) -> UnorderedSetMapper<ContractInfo<Self::Api>>;

    #[view(getChainFactoryAddress)]
    #[storage_mapper("chainFactories")]
    fn chain_factories(&self, shard_id: u32) -> SingleValueMapper<ManagedAddress>;

    #[view(getTokenHandlerAddress)]
    #[storage_mapper("tokenHadlersFactories")]
    fn token_handlers(&self, shard_id: u32) -> SingleValueMapper<ManagedAddress>;

    #[view(getDeployCost)]
    #[storage_mapper("deployCost")]
    fn deploy_cost(&self) -> SingleValueMapper<BigUint>;

    #[view(getAllChainIds)]
    #[storage_mapper("allChainIds")]
    fn chain_ids(&self) -> UnorderedSetMapper<ManagedBuffer>;
}
