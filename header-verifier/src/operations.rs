use error_messages::{
    CALLER_NOT_FROM_CURRENT_SOVEREIGN, CURRENT_OPERATION_ALREADY_IN_EXECUTION,
    CURRENT_OPERATION_NOT_REGISTERED, HASH_OF_HASHES_DOES_NOT_MATCH, INCORRECT_OPERATION_NONCE,
    OUTGOING_TX_HASH_ALREADY_REGISTERED, SETUP_PHASE_NOT_COMPLETED,
    VALIDATORS_ALREADY_REGISTERED_IN_EPOCH,
};
use structs::OperationHashStatus;

use crate::{
    checks,
    header_utils::{self, MAX_STORED_EPOCHS},
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
    + common_utils::CommonUtilsModule
{
    #[endpoint(registerBridgeOps)]
    fn register_bridge_operations(
        &self,
        signature: ManagedBuffer,
        hash_of_hashes: ManagedBuffer,
        pub_keys_bitmap: ManagedBuffer,
        epoch: u64,
        operations_hashes: MultiValueEncoded<ManagedBuffer>,
    ) {
        if !self.is_setup_phase_complete() {
            sc_panic!(SETUP_PHASE_NOT_COMPLETED);
        }

        let bls_pub_keys_mapper = self.bls_pub_keys(epoch);

        let mut hash_of_hashes_history_mapper = self.hash_of_hashes_history();

        if self.is_hash_of_hashes_registered(&hash_of_hashes, &hash_of_hashes_history_mapper) {
            sc_panic!(OUTGOING_TX_HASH_ALREADY_REGISTERED);
        }

        if self
            .calculate_and_check_transfers_hashes(&hash_of_hashes, operations_hashes.clone())
            .is_some()
        {
            sc_panic!(HASH_OF_HASHES_DOES_NOT_MATCH);
        }

        self.verify_bls(
            epoch,
            &signature,
            &hash_of_hashes,
            pub_keys_bitmap,
            &ManagedVec::from_iter(bls_pub_keys_mapper.iter()),
        );

        for operation_hash in operations_hashes {
            self.operation_hash_status(&hash_of_hashes, &operation_hash)
                .set(OperationHashStatus::NotLocked);
        }

        hash_of_hashes_history_mapper.insert(hash_of_hashes);
        self.last_operation_nonce().update(|nonce| *nonce += 1);
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

        self.verify_bls(
            epoch - 1, // Use the validator signatures from the last epoch
            &signature,
            &hash_of_hashes,
            pub_keys_bitmap,
            &ManagedVec::from_iter(bls_keys_previous_epoch.iter()),
        );

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

    // Add nonce increment
    #[endpoint(lockOperationHash)]
    fn lock_operation_hash(
        &self,
        hash_of_hashes: ManagedBuffer,
        operation_hash: ManagedBuffer,
        operation_nonce: u64,
    ) -> OptionalValue<ManagedBuffer> {
        if !self.is_caller_from_current_sovereign() {
            return OptionalValue::Some(CALLER_NOT_FROM_CURRENT_SOVEREIGN.into());
        }

        let last_nonce = self.last_operation_nonce().get();
        if operation_nonce <= last_nonce {
            return OptionalValue::Some(INCORRECT_OPERATION_NONCE.into());
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
