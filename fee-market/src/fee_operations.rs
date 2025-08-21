use structs::{aliases::GasLimit, fee::FinalPayment};

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
        self.distribute_fees_common_function(&hash_of_hashes, address_percentage_pairs);
    }

    #[payable("*")]
    #[endpoint(subtractFee)]
    fn subtract_fee(
        &self,
        original_caller: ManagedAddress,
        total_transfers: usize,
        opt_gas_limit: OptionalValue<GasLimit>,
    ) -> FinalPayment<Self::Api> {
        self.subtract_fee_common_function(original_caller, total_transfers, opt_gas_limit)
    }
}
