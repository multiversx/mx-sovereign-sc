use error_messages::{SETUP_PHASE_ALREADY_COMPLETED, SETUP_PHASE_NOT_COMPLETED};
use structs::{
    fee::{AddUsersToWhitelistOperation, RemoveUsersFromWhitelistOperation},
    generate_hash::GenerateHash,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeWhitelistModule:
    fee_common::storage::FeeCommonStorageModule
    + setup_phase::SetupPhaseModule
    + utils::UtilsModule
    + custom_events::CustomEventsModule
{
    #[only_owner]
    #[endpoint(addUsersToWhitelistSetupPhase)]
    fn add_users_to_whitelist_during_setup_phase(&self, users: MultiValueEncoded<ManagedAddress>) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

        self.users_whitelist().extend(users);
    }

    #[endpoint(addUsersToWhitelist)]
    fn add_users_to_whitelist(
        &self,
        hash_of_hashes: ManagedBuffer,
        operation: AddUsersToWhitelistOperation<Self::Api>,
    ) {
        let operation_hash = operation.generate_hash();
        if let Some(error_message) = self.validate_operation_hash(&operation_hash) {
            self.complete_operation(&hash_of_hashes, &operation_hash, Some(error_message));
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
        self.lock_operation_hash_wrapper(&hash_of_hashes, &operation_hash);
        self.users_whitelist().extend(operation.users);
        self.complete_operation(&hash_of_hashes, &operation_hash, None);
    }

    #[only_owner]
    #[endpoint(removeUsersFromWhitelistSetupPhase)]
    fn remove_users_from_whitelist_during_setup_phase(
        &self,
        users: MultiValueEncoded<ManagedAddress>,
    ) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

        for user in users {
            self.users_whitelist().swap_remove(&user);
        }
    }

    #[endpoint(removeUsersFromWhitelist)]
    fn remove_users_from_whitelist(
        &self,
        hash_of_hashes: ManagedBuffer,
        operation: RemoveUsersFromWhitelistOperation<Self::Api>,
    ) {
        let operation_hash = operation.generate_hash();
        if let Some(error_message) = self.validate_operation_hash(&operation_hash) {
            self.complete_operation(&hash_of_hashes, &operation_hash, Some(error_message));
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
        self.lock_operation_hash_wrapper(&hash_of_hashes, &operation_hash);

        for user in &operation.users {
            self.users_whitelist().swap_remove(&user);
        }

        self.complete_operation(&hash_of_hashes, &operation_hash, None);
    }
}
