use structs::fee::{AddressPercentagePair, FeeStruct};
multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FeeOperationsModule:
    tx_nonce::TxNonceModule
    + custom_events::CustomEventsModule
    + fee_common::helpers::FeeCommonHelpersModule
    + fee_common::storage::FeeCommonStorageModule
    + common_utils::CommonUtilsModule
{
    #[only_owner]
    #[endpoint(setFee)]
    fn set_fee(&self, fee_struct: FeeStruct<Self::Api>) {
        self.set_fee_event(fee_struct, self.get_and_save_next_tx_id());
    }

    #[only_owner]
    #[endpoint(removeFee)]
    fn remove_fee(&self, token_id: EgldOrEsdtTokenIdentifier<Self::Api>) {
        self.remove_fee_event(token_id, self.get_and_save_next_tx_id());
    }

    #[only_owner]
    #[endpoint(distributeFees)]
    fn distribute_fees(
        &self,
        address_percentage_pairs: ManagedVec<AddressPercentagePair<Self::Api>>,
    ) {
        self.distribute_fees_event(address_percentage_pairs, self.get_and_save_next_tx_id());
    }

    #[only_owner]
    #[endpoint(addUsersToFeeWhitelist)]
    fn add_users_to_fee_whitelist(&self, users: ManagedVec<ManagedAddress<Self::Api>>) {
        self.add_users_to_fee_whitelist_event(users, self.get_and_save_next_tx_id());
    }

    #[only_owner]
    #[endpoint(removeUsersFromFeeWhitelist)]
    fn remove_users_from_fee_whitelist(&self, users: ManagedVec<ManagedAddress<Self::Api>>) {
        self.remove_users_from_fee_whitelist_event(users, self.get_and_save_next_tx_id());
    }
}
