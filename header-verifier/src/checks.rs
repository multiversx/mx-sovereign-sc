use error_messages::{
    BITMAP_LEN_DOES_NOT_MATCH_BLS_KEY_LEN, CHAIN_CONFIG_SETUP_PHASE_NOT_COMPLETE,
    CURRENT_OPERATION_NOT_REGISTERED, HASH_OF_HASHES_DOES_NOT_MATCH,
    OUTGOING_TX_HASH_ALREADY_REGISTERED, VALIDATORS_ALREADY_REGISTERED_IN_EPOCH,
};

use crate::header_utils::OperationHashStatus;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait HeaderVerifierChecksModule:
    crate::storage::HeaderVerifierStorageModule
    + custom_events::CustomEventsModule
    + setup_phase::SetupPhaseModule
    + utils::UtilsModule
{
    fn require_bls_pub_keys_empty(&self, epoch: u64) {
        require!(
            self.bls_pub_keys(epoch).is_empty(),
            VALIDATORS_ALREADY_REGISTERED_IN_EPOCH
        );
    }

    fn require_chain_config_setup_complete(&self, chain_config_address: &ManagedAddress) {
        require!(
            self.chain_config_setup_phase_complete(chain_config_address.clone())
                .get(),
            CHAIN_CONFIG_SETUP_PHASE_NOT_COMPLETE
        );
    }

    fn require_bitmap_and_bls_same_length(&self, bitmap_len: usize, bls_len: usize) {
        require!(bitmap_len == bls_len, BITMAP_LEN_DOES_NOT_MATCH_BLS_KEY_LEN);
    }

    fn require_hash_of_hashes_not_registered(
        &self,
        hash_of_hashes: &ManagedBuffer,
        history_mapper: &UnorderedSetMapper<ManagedBuffer>,
    ) {
        require!(
            !history_mapper.contains(hash_of_hashes),
            OUTGOING_TX_HASH_ALREADY_REGISTERED
        );
    }

    fn require_operation_hash_registered(
        &self,
        hash_status_mapper: &SingleValueMapper<OperationHashStatus>,
    ) {
        require!(
            !hash_status_mapper.is_empty(),
            CURRENT_OPERATION_NOT_REGISTERED
        );
    }

    fn require_matching_hash_of_hashes(
        &self,
        hash_of_hashes: &ManagedBuffer,
        computed_hash_of_hashes: &ManagedBuffer,
    ) {
        require!(
            computed_hash_of_hashes.eq(hash_of_hashes),
            HASH_OF_HASHES_DOES_NOT_MATCH
        );
    }
}
