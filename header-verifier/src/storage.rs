use structs::{configs::SovereignConfig, forge::ContractInfo};

use crate::header_utils::OperationHashStatus;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait HeaderVerifierStorageModule {
    #[storage_mapper("blsPubKeys")]
    fn bls_pub_keys(&self, epoch: u64) -> SetMapper<ManagedBuffer>;

    #[storage_mapper_from_address("blsKeyToId")]
    fn bls_key_to_id_mapper(
        &self,
        sc_address: ManagedAddress,
        bls_key: &ManagedBuffer,
    ) -> SingleValueMapper<BigUint<Self::Api>, ManagedAddress>;

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

    #[storage_mapper("operationHashStatus")]
    fn operation_hash_status(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_hash: &ManagedBuffer,
    ) -> SingleValueMapper<OperationHashStatus>;

    #[storage_mapper("hashOfHashesHistory")]
    fn hash_of_hashes_history(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("esdtSafeAddress")]
    fn esdt_safe_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("chainConfigAddress")]
    fn chain_config_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("sovereignContracts")]
    fn sovereign_contracts(&self) -> UnorderedSetMapper<ContractInfo<Self::Api>>;

    #[storage_mapper_from_address("sovereignConfig")]
    fn sovereign_config(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<SovereignConfig<Self::Api>, ManagedAddress>;
}
