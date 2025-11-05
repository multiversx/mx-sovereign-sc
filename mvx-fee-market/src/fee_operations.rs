use error_messages::{SETUP_PHASE_ALREADY_COMPLETED, SETUP_PHASE_NOT_COMPLETED};
use structs::{
    fee::{DistributeFeesOperation, FeeStruct, RemoveFeeOperation, SetFeeOperation},
    generate_hash::GenerateHash,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeOperationsModule:
    setup_phase::SetupPhaseModule
    + custom_events::CustomEventsModule
    + common_utils::CommonUtilsModule
    + fee_common::storage::FeeCommonStorageModule
    + fee_common::helpers::FeeCommonHelpersModule
    + fee_common::endpoints::FeeCommonEndpointsModule
{
    #[endpoint(distributeFees)]
    fn distribute_fees(
        &self,
        hash_of_hashes: ManagedBuffer,
        operation: DistributeFeesOperation<Self::Api>,
    ) {
        let operation_hash = operation.generate_hash();
        if let Some(lock_operation_error) =
            self.lock_operation_hash_wrapper(&hash_of_hashes, &operation_hash, operation.nonce)
        {
            self.complete_operation(&hash_of_hashes, &operation_hash, Some(lock_operation_error));
            return;
        }
        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }
        if let Some(err_msg) = self.validate_percentage_sum(&operation.pairs) {
            self.complete_operation(&hash_of_hashes, &operation_hash, Some(err_msg));
            return;
        }

        self.distribute_fees_and_reset(&operation.pairs);
        self.complete_operation(&hash_of_hashes, &operation_hash, None);
    }

    #[only_owner]
    #[endpoint(removeFeeDuringSetupPhase)]
    fn remove_fee_during_setup_phase(&self, base_token: EgldOrEsdtTokenIdentifier<Self::Api>) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

        self.remove_fee_from_storage(&base_token);
    }

    #[endpoint(removeFee)]
    fn remove_fee(
        &self,
        hash_of_hashes: ManagedBuffer,
        remove_fee_operation: RemoveFeeOperation<Self::Api>,
    ) {
        let token_id_hash = remove_fee_operation.generate_hash();
        if let Some(lock_operation_error) = self.lock_operation_hash_wrapper(
            &hash_of_hashes,
            &token_id_hash,
            remove_fee_operation.nonce,
        ) {
            self.complete_operation(&hash_of_hashes, &token_id_hash, Some(lock_operation_error));
            return;
        }
        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &token_id_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }
        self.remove_fee_from_storage(&remove_fee_operation.token_id);
        self.complete_operation(&hash_of_hashes, &token_id_hash, None);
    }

    #[only_owner]
    #[endpoint(setFeeDuringSetupPhase)]
    fn set_fee_during_setup_phase(&self, fee_struct: FeeStruct<Self::Api>) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

        if let Some(set_fee_error_msg) = self.set_fee_in_storage(&fee_struct) {
            sc_panic!(set_fee_error_msg);
        }
    }

    #[endpoint(setFee)]
    fn set_fee(
        &self,
        hash_of_hashes: ManagedBuffer,
        set_fee_operation: SetFeeOperation<Self::Api>,
    ) {
        let fee_hash = set_fee_operation.generate_hash();
        if let Some(lock_operation_error) =
            self.lock_operation_hash_wrapper(&hash_of_hashes, &fee_hash, set_fee_operation.nonce)
        {
            self.complete_operation(&hash_of_hashes, &fee_hash, Some(lock_operation_error));
            return;
        }
        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &fee_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }
        if let Some(set_fee_error_msg) = self.set_fee_in_storage(&set_fee_operation.fee_struct) {
            self.complete_operation(&hash_of_hashes, &fee_hash, Some(set_fee_error_msg.into()));
            return;
        }

        self.complete_operation(&hash_of_hashes, &fee_hash, None);
    }
}
