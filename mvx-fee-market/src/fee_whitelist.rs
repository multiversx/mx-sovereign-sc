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
    + common_utils::CommonUtilsModule
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
        add_to_whitelist_operation: AddUsersToWhitelistOperation<Self::Api>,
    ) {
        let operation_hash = add_to_whitelist_operation.generate_hash();
        if let Some(lock_operation_error) = self.lock_operation_hash_wrapper(
            &hash_of_hashes,
            &operation_hash,
            add_to_whitelist_operation.nonce,
        ) {
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
        self.users_whitelist()
            .extend(add_to_whitelist_operation.users);
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
        remove_from_whitelist_operation: RemoveUsersFromWhitelistOperation<Self::Api>,
    ) {
        let operation_hash = remove_from_whitelist_operation.generate_hash();
        if let Some(lock_operation_error) = self.lock_operation_hash_wrapper(
            &hash_of_hashes,
            &operation_hash,
            remove_from_whitelist_operation.nonce,
        ) {
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

        for user in &remove_from_whitelist_operation.users {
            self.users_whitelist().swap_remove(&user);
        }

        self.complete_operation(&hash_of_hashes, &operation_hash, None);
    }
}
