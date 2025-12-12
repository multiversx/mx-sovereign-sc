use multiversx_sc::{
    imports::{SingleValueMapper, UnorderedSetMapper},
    types::ManagedBuffer,
};
use structs::forge::ContractInfo;

pub type ChainId<M> = ManagedBuffer<M>;

#[multiversx_sc::module]
pub trait StorageModule {
    #[storage_mapper("sovereignsMapper")]
    fn sovereigns_mapper(
        &self,
        sovereign_creator: &ManagedAddress,
    ) -> SingleValueMapper<ChainId<Self::Api>>;

    #[view(getDeployedSovereignContracts)]
    #[storage_mapper("sovereignDeployedContracts")]
    fn sovereign_deployed_contracts(
        &self,
        chain_id: &ChainId<Self::Api>,
    ) -> UnorderedSetMapper<ContractInfo<Self::Api>>;

    #[view(getTrustedTokens)]
    #[storage_mapper("trustedTokens")]
    fn trusted_tokens(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[view(getSovereignSetupPhase)]
    #[storage_mapper("sovereignSetupPhase")]
    fn sovereign_setup_phase(&self, chain_id: &ChainId<Self::Api>) -> SingleValueMapper<bool>;

    #[view(getChainFactoryAddress)]
    #[storage_mapper("chainFactories")]
    fn chain_factories(&self, shard_id: u32) -> SingleValueMapper<ManagedAddress>;

    #[view(getDeployCost)]
    #[storage_mapper("deployCost")]
    fn deploy_cost(&self) -> SingleValueMapper<BigUint>;

    #[view(getAllChainIds)]
    #[storage_mapper("allChainIds")]
    fn chain_ids(&self) -> UnorderedSetMapper<ManagedBuffer>;
}
