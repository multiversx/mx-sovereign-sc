use error_messages::ITEM_NOT_IN_LIST;

use crate::storage;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeWhitelistModule: storage::FeeStorageModule {
    #[only_owner]
    #[endpoint(addUsersToWhitelist)]
    fn add_users_to_whitelist(&self, users: MultiValueEncoded<ManagedAddress>) {
        self.users_whitelist().extend(users);
    }

    #[only_owner]
    #[endpoint(removeUsersFromWhitelist)]
    fn remove_users_from_whitelist(&self, users: MultiValueEncoded<ManagedAddress>) {
        for user in users {
            let was_removed = self.users_whitelist().swap_remove(&user);
            require!(was_removed, ITEM_NOT_IN_LIST);
        }
    }
}
