use error_messages::CHAIN_CONFIG_SETUP_PHASE_NOT_COMPLETE;
use structs::OperationHashStatus;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait HeaderVerifierChecksModule:
    crate::storage::HeaderVerifierStorageModule
    + custom_events::CustomEventsModule
    + setup_phase::SetupPhaseModule
    + common_utils::CommonUtilsModule
{
    fn is_bls_pub_keys_empty(&self, epoch: u64) -> bool {
        self.bls_pub_keys(epoch).is_empty()
    }

    fn require_chain_config_setup_complete(&self, chain_config_address: &ManagedAddress) {
        require!(
            self.chain_config_setup_phase_complete(chain_config_address.clone())
                .get(),
            CHAIN_CONFIG_SETUP_PHASE_NOT_COMPLETE
        );
    }

    fn is_hash_of_hashes_registered(
        &self,
        hash_of_hashes: &ManagedBuffer,
        history_mapper: &UnorderedSetMapper<ManagedBuffer>,
    ) -> bool {
        history_mapper.contains(hash_of_hashes)
    }

    fn is_hash_status_mapper_empty(
        &self,
        hash_status_mapper: &SingleValueMapper<OperationHashStatus>,
    ) -> bool {
        hash_status_mapper.is_empty()
    }

    fn are_hash_of_hashes_matching(
        &self,
        hash_of_hashes: &ManagedBuffer,
        computed_hash_of_hashes: &ManagedBuffer,
    ) -> bool {
        computed_hash_of_hashes.eq(hash_of_hashes)
    }
}
