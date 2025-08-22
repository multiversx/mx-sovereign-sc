use error_messages::{ERROR_AT_ENCODING, SETUP_PHASE_ALREADY_COMPLETED};
use structs::{fee::FeeStruct, generate_hash::GenerateHash};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeTypeModule:
    utils::UtilsModule
    + setup_phase::SetupPhaseModule
    + custom_events::CustomEventsModule
    + fee_common::storage::FeeCommonStorageModule
    + fee_common::helpers::FeeCommonHelpersModule
{
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
