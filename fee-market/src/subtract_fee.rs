use structs::{aliases::GasLimit, fee::FinalPayment};

use crate::fee_whitelist;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait SubtractFeeModule:
    crate::fee_type::FeeTypeModule
    + fee_common::storage::FeeCommonStorageModule
    + fee_common::endpoints::FeeCommonEndpointsModule
    + fee_common::helpers::FeeCommonHelpersModule
    + utils::UtilsModule
    + setup_phase::SetupPhaseModule
    + custom_events::CustomEventsModule
    + fee_whitelist::FeeWhitelistModule
{
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
