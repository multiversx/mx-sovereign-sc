use error_messages::{
    BITMAP_LEN_DOES_NOT_MATCH_BLS_KEY_LEN, CURRENT_OPERATION_ALREADY_IN_EXECUTION,
    OUTGOING_TX_HASH_ALREADY_REGISTERED,
};

use crate::{
    checks, storage,
    utils::{self, OperationHashStatus, EPOCH_RANGE},
};
multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait HeaderVerifierOperationsModule:
    utils::HeaderVerifierUtilsModule
    + storage::HeaderVerifierStorageModule
    + checks::HeaderVerifierChecksModule
    + events::EventsModule
    + setup_phase::SetupPhaseModule
{
    #[endpoint(registerBridgeOps)]
    fn register_bridge_operations(
        &self,
        signature: ManagedBuffer,
        bridge_operations_hash: ManagedBuffer,
        pub_keys_bitmap: ManagedBuffer,
        epoch: u64,
        operations_hashes: MultiValueEncoded<ManagedBuffer>,
    ) {
        self.require_setup_complete();
        let bls_pub_keys_mapper = self.bls_pub_keys(epoch);
        require!(
            pub_keys_bitmap.len() == bls_pub_keys_mapper.len(),
            BITMAP_LEN_DOES_NOT_MATCH_BLS_KEY_LEN
        );

        let mut hash_of_hashes_history_mapper = self.hash_of_hashes_history();

        require!(
            !hash_of_hashes_history_mapper.contains(&bridge_operations_hash),
            OUTGOING_TX_HASH_ALREADY_REGISTERED
        );

        self.verify_bls(
            &signature,
            &bridge_operations_hash,
            pub_keys_bitmap,
            &ManagedVec::from_iter(bls_pub_keys_mapper.iter()),
        );

        self.calculate_and_check_transfers_hashes(
            &bridge_operations_hash,
            operations_hashes.clone(),
        );

        for operation_hash in operations_hashes {
            self.operation_hash_status(&bridge_operations_hash, &operation_hash)
                .set(OperationHashStatus::NotLocked);
        }

        hash_of_hashes_history_mapper.insert(bridge_operations_hash);
    }

    #[endpoint(changeValidatorSet)]
    fn change_validator_set(
        &self,
        signature: ManagedBuffer,
        bridge_operations_hash: ManagedBuffer,
        operation_hash: ManagedBuffer,
        pub_keys_bitmap: ManagedBuffer,
        epoch: u64,
        pub_keys_id: MultiValueEncoded<BigUint<Self::Api>>,
    ) {
        self.require_setup_complete();
        self.require_bls_pub_keys_empty(epoch);

        let bls_keys_previous_epoch = self.bls_pub_keys(epoch - 1);

        self.require_bitmap_and_bls_same_length(
            pub_keys_bitmap.len(),
            bls_keys_previous_epoch.len(),
        );

        let mut hash_of_hashes_history_mapper = self.hash_of_hashes_history();

        self.require_hash_of_hashes_not_registered(
            &bridge_operations_hash,
            &hash_of_hashes_history_mapper,
        );

        self.verify_bls(
            &signature,
            &bridge_operations_hash,
            pub_keys_bitmap,
            &ManagedVec::from_iter(bls_keys_previous_epoch.iter()),
        );

        let mut operations_hashes = MultiValueEncoded::new();
        operations_hashes.push(operation_hash.clone());

        self.calculate_and_check_transfers_hashes(
            &bridge_operations_hash,
            operations_hashes.clone(),
        );

        if epoch > EPOCH_RANGE && !self.bls_pub_keys(epoch - EPOCH_RANGE).is_empty() {
            self.bls_pub_keys(epoch - EPOCH_RANGE).clear();
        }

        let new_bls_keys = self.get_bls_keys_by_id(pub_keys_id);
        self.bls_pub_keys(epoch).extend(new_bls_keys);

        hash_of_hashes_history_mapper.insert(bridge_operations_hash.clone());
        self.execute_bridge_operation_event(&bridge_operations_hash, &operation_hash);
    }

    #[endpoint(removeExecutedHash)]
    fn remove_executed_hash(&self, hash_of_hashes: &ManagedBuffer, operation_hash: &ManagedBuffer) {
        self.require_caller_is_from_current_sovereign();

        self.operation_hash_status(hash_of_hashes, operation_hash)
            .clear();
    }

    #[endpoint(lockOperationHash)]
    fn lock_operation_hash(&self, hash_of_hashes: ManagedBuffer, operation_hash: ManagedBuffer) {
        self.require_caller_is_from_current_sovereign();

        let operation_hash_status_mapper =
            self.operation_hash_status(&hash_of_hashes, &operation_hash);

        self.require_operation_hash_registered(&operation_hash_status_mapper);

        let is_hash_in_execution = operation_hash_status_mapper.get();
        match is_hash_in_execution {
            OperationHashStatus::NotLocked => {
                operation_hash_status_mapper.set(OperationHashStatus::Locked)
            }
            OperationHashStatus::Locked => {
                sc_panic!(CURRENT_OPERATION_ALREADY_IN_EXECUTION)
            }
        }
    }
}
