use structs::fee::FeeStruct;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeOperationsModule:
    custom_events::CustomEventsModule
    + common_utils::CommonUtilsModule
    + fee_common::storage::FeeCommonStorageModule
    + fee_common::helpers::FeeCommonHelpersModule
    + fee_common::endpoints::FeeCommonEndpointsModule
{
    #[only_owner]
    #[endpoint(distributeFees)]
    fn distribute_fees(
        &self,
        address_percentage_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
    ) {
        let pairs = self.parse_pairs(address_percentage_pairs);
        if let Some(percentage_validation_err) = self.validate_percentage_sum(&pairs) {
            sc_panic!(percentage_validation_err);
        }

        self.distribute_fees_and_reset(&pairs);
    }

    #[only_owner]
    #[endpoint(removeFee)]
    fn remove_fee(&self, token_id: EgldOrEsdtTokenIdentifier) {
        self.remove_fee_from_storage(&token_id);
    }

    #[only_owner]
    #[endpoint(setFee)]
    fn set_fee(&self, fee_struct: FeeStruct<Self::Api>) {
        if let Some(set_fee_error_msg) = self.set_fee_in_storage(&fee_struct) {
            sc_panic!(set_fee_error_msg);
        }
    }
}
