use structs::{aliases::TxNonce, forge::ContractInfo, OperationHashStatus};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait HeaderVerifierStorageModule {
    #[storage_mapper("blsPubKeys")]
    fn bls_pub_keys(&self, epoch: u64) -> SetMapper<ManagedBuffer>;

    #[storage_mapper_from_address("blsKeysMap")]
    fn bls_keys_map(
        &self,
        sc_address: ManagedAddress,
    ) -> MapMapper<BigUint<Self::Api>, ManagedBuffer, ManagedAddress>;

    #[storage_mapper_from_address("setupPhaseComplete")]
    fn chain_config_setup_phase_complete(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<bool, ManagedAddress>;

    #[view(operationHashStatus)]
    #[storage_mapper("operationHashStatus")]
    fn operation_hash_status(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_hash: &ManagedBuffer,
    ) -> SingleValueMapper<OperationHashStatus>;

    #[storage_mapper("hashOfHashesHistory")]
    fn hash_of_hashes_history(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("sovereignContracts")]
    fn sovereign_contracts(&self) -> UnorderedSetMapper<ContractInfo<Self::Api>>;

    #[storage_mapper("operationNonce")]
    fn current_execution_nonce(&self) -> SingleValueMapper<TxNonce>;
}
