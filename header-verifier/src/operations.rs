use error_messages::{
    BITMAP_LEN_DOES_NOT_MATCH_BLS_KEY_LEN, BLS_SIGNATURE_NOT_VALID,
    CALLER_NOT_FROM_CURRENT_SOVEREIGN, CURRENT_OPERATION_ALREADY_IN_EXECUTION,
    CURRENT_OPERATION_NOT_REGISTERED, HASH_OF_HASHES_DOES_NOT_MATCH,
    OUTGOING_TX_HASH_ALREADY_REGISTERED, SETUP_PHASE_NOT_COMPLETED,
    VALIDATORS_ALREADY_REGISTERED_IN_EPOCH,
};

use crate::{
    checks,
    header_utils::{self, OperationHashStatus, MAX_STORED_EPOCHS},
    storage,
};
multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait HeaderVerifierOperationsModule:
    header_utils::HeaderVerifierUtilsModule
    + storage::HeaderVerifierStorageModule
    + checks::HeaderVerifierChecksModule
    + custom_events::CustomEventsModule
    + setup_phase::SetupPhaseModule
    + utils::UtilsModule
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
        if !self.is_setup_phase_complete() {
            sc_panic!(SETUP_PHASE_NOT_COMPLETED);
        }

        let bls_pub_keys_mapper = self.bls_pub_keys(epoch);

        if !self.is_bitmap_and_bls_same_length(pub_keys_bitmap.len(), bls_pub_keys_mapper.len()) {
            sc_panic!(BITMAP_LEN_DOES_NOT_MATCH_BLS_KEY_LEN);
        }

        let mut hash_of_hashes_history_mapper = self.hash_of_hashes_history();

        if self
            .is_hash_of_hashes_registered(&bridge_operations_hash, &hash_of_hashes_history_mapper)
        {
            sc_panic!(OUTGOING_TX_HASH_ALREADY_REGISTERED);
        }

        if self
            .calculate_and_check_transfers_hashes(
                &bridge_operations_hash,
                operations_hashes.clone(),
            )
            .is_some()
        {
            sc_panic!(HASH_OF_HASHES_DOES_NOT_MATCH);
        }

        if self
            .verify_bls(
                epoch,
                &signature,
                &bridge_operations_hash,
                pub_keys_bitmap,
                &ManagedVec::from_iter(bls_pub_keys_mapper.iter()),
            )
            .is_some()
        {
            sc_panic!(BLS_SIGNATURE_NOT_VALID);
        }

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
        hash_of_hashes: ManagedBuffer,
        operation_hash: ManagedBuffer,
        pub_keys_bitmap: ManagedBuffer,
        epoch: u64,
        pub_keys_id: MultiValueEncoded<BigUint<Self::Api>>,
    ) {
        if !self.is_setup_phase_complete() {
            self.execute_bridge_operation_event(
                &hash_of_hashes,
                &operation_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );

            return;
        }
        if !self.is_bls_pub_keys_empty(epoch) {
            self.execute_bridge_operation_event(
                &hash_of_hashes,
                &operation_hash,
                Some(VALIDATORS_ALREADY_REGISTERED_IN_EPOCH.into()),
            );

            return;
        }
        let bls_keys_previous_epoch = self.bls_pub_keys(epoch - 1);
        if !self.is_bitmap_and_bls_same_length(pub_keys_bitmap.len(), bls_keys_previous_epoch.len())
        {
            self.execute_bridge_operation_event(
                &hash_of_hashes,
                &operation_hash,
                Some(BITMAP_LEN_DOES_NOT_MATCH_BLS_KEY_LEN.into()),
            );

            return;
        }
        let mut hash_of_hashes_history_mapper = self.hash_of_hashes_history();
        if self.is_hash_of_hashes_registered(&hash_of_hashes, &hash_of_hashes_history_mapper) {
            self.execute_bridge_operation_event(
                &hash_of_hashes,
                &operation_hash,
                Some(OUTGOING_TX_HASH_ALREADY_REGISTERED.into()),
            );

            return;
        }

        let mut operations_hashes = MultiValueEncoded::new();
        operations_hashes.push(operation_hash.clone());

        if let Some(error_message) =
            self.calculate_and_check_transfers_hashes(&hash_of_hashes, operations_hashes.clone())
        {
            self.execute_bridge_operation_event(
                &hash_of_hashes,
                &operation_hash,
                Some(error_message),
            );
            return;
        }

        if self
            .verify_bls(
                epoch - 1, // Use the validator signatures from the last epoch
                &signature,
                &hash_of_hashes,
                pub_keys_bitmap,
                &ManagedVec::from_iter(bls_keys_previous_epoch.iter()),
            )
            .is_some()
        {
            self.execute_bridge_operation_event(
                &hash_of_hashes,
                &operation_hash,
                Some(BLS_SIGNATURE_NOT_VALID.into()),
            );
            return;
        }

        if epoch >= MAX_STORED_EPOCHS && !self.bls_pub_keys(epoch - MAX_STORED_EPOCHS).is_empty() {
            self.bls_pub_keys(epoch - MAX_STORED_EPOCHS).clear();
        }

        let new_bls_keys = self.get_bls_keys_by_id(pub_keys_id);
        self.bls_pub_keys(epoch).extend(new_bls_keys);

        hash_of_hashes_history_mapper.insert(hash_of_hashes.clone());
        self.execute_bridge_operation_event(&hash_of_hashes, &operation_hash, None);
    }

    #[endpoint(removeExecutedHash)]
    fn remove_executed_hash(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_hash: &ManagedBuffer,
    ) -> OptionalValue<ManagedBuffer> {
        if !self.is_caller_from_current_sovereign() {
            return OptionalValue::Some(CALLER_NOT_FROM_CURRENT_SOVEREIGN.into());
        }

        self.operation_hash_status(hash_of_hashes, operation_hash)
            .clear();

        OptionalValue::None
    }

    #[endpoint(lockOperationHash)]
    fn lock_operation_hash(
        &self,
        hash_of_hashes: ManagedBuffer,
        operation_hash: ManagedBuffer,
    ) -> OptionalValue<ManagedBuffer> {
        if !self.is_caller_from_current_sovereign() {
            return OptionalValue::Some(CALLER_NOT_FROM_CURRENT_SOVEREIGN.into());
        }

        let operation_hash_status_mapper =
            self.operation_hash_status(&hash_of_hashes, &operation_hash);

        if self.is_hash_status_mapper_empty(&operation_hash_status_mapper) {
            return OptionalValue::Some(CURRENT_OPERATION_NOT_REGISTERED.into());
        }

        let is_hash_in_execution = operation_hash_status_mapper.get();
        match is_hash_in_execution {
            OperationHashStatus::NotLocked => {
                operation_hash_status_mapper.set(OperationHashStatus::Locked)
            }
            OperationHashStatus::Locked => {
                return OptionalValue::Some(CURRENT_OPERATION_ALREADY_IN_EXECUTION.into());
            }
        }

        OptionalValue::None
    }
}
