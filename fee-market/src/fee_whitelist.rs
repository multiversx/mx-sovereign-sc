use error_messages::{ITEM_NOT_IN_LIST, SETUP_PHASE_ALREADY_COMPLETED};

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
        users: MultiValueEncoded<ManagedAddress>,
    ) {
        self.require_setup_complete();

        let users_hash = self.get_users_aggregated_hash(users.clone());

        self.users_whitelist().extend(users);

        self.complete_operation(&hash_of_hashes, &users_hash, None);
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
        users: MultiValueEncoded<ManagedAddress>,
    ) {
        self.require_setup_complete();

        let users_hash = self.get_users_aggregated_hash(users.clone());

        for user in users {
            self.users_whitelist().swap_remove(&user);
        }

        self.complete_operation(&hash_of_hashes, &users_hash, None);
    }

    fn get_users_aggregated_hash(&self, users: MultiValueEncoded<ManagedAddress>) -> ManagedBuffer {
        let mut aggregated_hashes = ManagedBuffer::new();

        for user in users {
            let user_buffer = user.as_managed_buffer();
            let user_hash_byte_array = self.crypto().sha256(user_buffer);

            aggregated_hashes.append(user_hash_byte_array.as_managed_buffer());
        }

        let users_aggregated_byte_array = self.crypto().sha256(aggregated_hashes);
        users_aggregated_byte_array.as_managed_buffer().clone()
    }
}
