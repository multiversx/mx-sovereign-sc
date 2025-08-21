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
}
