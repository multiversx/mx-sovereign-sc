use error_messages::{SETUP_PHASE_ALREADY_COMPLETED, SETUP_PHASE_NOT_COMPLETED};
use structs::{
    fee::{DistributeFeesOperation, FeeStruct},
    generate_hash::GenerateHash,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const TOTAL_PERCENTAGE: usize = 10_000;

#[multiversx_sc::module]
pub trait FeeOperationsModule:
    setup_phase::SetupPhaseModule
    + custom_events::CustomEventsModule
    + utils::UtilsModule
    + fee_common::storage::FeeCommonStorageModule
    + fee_common::helpers::FeeCommonHelpersModule
    + fee_common::endpoints::FeeCommonEndpointsModule
{
    #[only_owner]
    #[endpoint(distributeFees)]
    fn distribute_fees(
        &self,
        hash_of_hashes: ManagedBuffer,
        operation: DistributeFeesOperation<Self::Api>,
    ) {
        let operation_hash = operation.generate_hash();
        if let Some(error_message) = self.validate_operation_hash(&operation_hash) {
            self.complete_operation(&hash_of_hashes, &operation_hash, Some(error_message));
            return;
        };
        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }
        self.lock_operation_hash_wrapper(&hash_of_hashes, &operation_hash);

        if let Some(err_msg) = self.validate_percentage_sum(&operation.pairs) {
            self.complete_operation(&hash_of_hashes, &operation_hash, Some(err_msg));
            return;
        }

        self.distribute_token_fees(&operation.pairs);
        self.tokens_for_fees().clear();
        self.complete_operation(&hash_of_hashes, &operation_hash, None);
    }

    #[only_owner]
    #[endpoint(removeFeeDuringSetupPhase)]
    fn remove_fee_during_setup_phase(&self, base_token: TokenIdentifier) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

        self.token_fee(&base_token).clear();
        self.fee_enabled().set(false);
    }

    #[endpoint(removeFee)]
    fn remove_fee(&self, hash_of_hashes: ManagedBuffer, token_id: TokenIdentifier) {
        let token_id_hash = token_id.generate_hash();
        if let Some(err_msg) = self.validate_operation_hash(&token_id_hash) {
            self.complete_operation(&hash_of_hashes, &token_id_hash, Some(err_msg));
            return;
        };
        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &token_id_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }

        self.lock_operation_hash_wrapper(&hash_of_hashes, &token_id_hash);
        self.token_fee(&token_id).clear();
        self.fee_enabled().set(false);
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
    fn set_fee(&self, hash_of_hashes: ManagedBuffer, fee_struct: FeeStruct<Self::Api>) {
        let fee_hash = fee_struct.generate_hash();
        if let Some(err_msg) = self.validate_operation_hash(&fee_hash) {
            self.complete_operation(&hash_of_hashes, &fee_hash, Some(err_msg));
            return;
        };
        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &fee_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }

        self.lock_operation_hash_wrapper(&hash_of_hashes, &fee_hash);

        if let Some(set_fee_error_msg) = self.set_fee_in_storage(&fee_struct) {
            self.complete_operation(&hash_of_hashes, &fee_hash, Some(set_fee_error_msg.into()));
            return;
        }

        self.complete_operation(&hash_of_hashes, &fee_hash, None);
    }
}
