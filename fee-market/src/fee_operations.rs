use error_messages::{ERROR_AT_ENCODING, SETUP_PHASE_ALREADY_COMPLETED};
use structs::{fee::FeeStruct, generate_hash::GenerateHash};

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
        address_percentage_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
    ) {
        self.require_setup_complete();
        let pairs = self.parse_pairs(address_percentage_pairs);
        let opt_pairs_hash = self.generate_pairs_hash(&pairs, &hash_of_hashes);
        if opt_pairs_hash.is_none() {
            return;
        }
        let pairs_hash = opt_pairs_hash.unwrap();

        if let Some(pairs_validation_error) = self.validate_pairs(&pairs) {
            self.complete_operation(&hash_of_hashes, &pairs_hash, Some(pairs_validation_error));
            return;
        }

        let pairs_hash = self.generate_pairs_hash(&pairs, &hash_of_hashes);
        if pairs_hash.is_none() {
            return;
        }
        let pairs_hash = pairs_hash.unwrap();
        self.lock_operation_hash(&hash_of_hashes, &pairs_hash);

        if let Some(err_msg) = self.validate_percentage_sum(&pairs) {
            self.complete_operation(&hash_of_hashes, &pairs_hash, Some(err_msg));
            return;
        }

        self.distribute_token_fees(&pairs);

        self.tokens_for_fees().clear();

        self.complete_operation(&hash_of_hashes, &pairs_hash, None);
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
        self.require_setup_complete();

        let token_id_hash = token_id.generate_hash();
        if token_id_hash.is_empty() {
            self.complete_operation(
                &hash_of_hashes,
                &token_id_hash,
                Some(ManagedBuffer::from(ERROR_AT_ENCODING)),
            );
            return;
        };

        self.lock_operation_hash(&hash_of_hashes, &token_id_hash);

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
        self.require_setup_complete();

        let fee_hash = fee_struct.generate_hash();
        if fee_hash.is_empty() {
            self.complete_operation(
                &hash_of_hashes,
                &fee_hash,
                Some(ManagedBuffer::from(ERROR_AT_ENCODING)),
            );
            return;
        };

        self.lock_operation_hash(&hash_of_hashes, &fee_hash);

        if let Some(set_fee_error_msg) = self.set_fee_in_storage(&fee_struct) {
            self.complete_operation(
                &hash_of_hashes,
                &fee_hash,
                Some(ManagedBuffer::from(set_fee_error_msg)),
            );
            return;
        }

        self.complete_operation(&hash_of_hashes, &fee_hash, None);
    }
}
