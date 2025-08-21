multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const TOTAL_PERCENTAGE: usize = 10_000;

#[multiversx_sc::module]
pub trait FeeOperationsModule:
    custom_events::CustomEventsModule
    + utils::UtilsModule
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

        self.distribute_token_fees(&pairs);
        self.tokens_for_fees().clear();
    }
}
