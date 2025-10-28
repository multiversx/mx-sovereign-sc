#![no_std]

use structs::fee::FeeStruct;

multiversx_sc::imports!();

pub mod fee_operations;
pub mod fee_whitelist;

#[multiversx_sc::contract]
pub trait MvxFeeMarket:
    common_utils::CommonUtilsModule
    + setup_phase::SetupPhaseModule
    + custom_events::CustomEventsModule
    + fee_operations::FeeOperationsModule
    + fee_common::storage::FeeCommonStorageModule
    + fee_common::endpoints::FeeCommonEndpointsModule
    + fee_common::helpers::FeeCommonHelpersModule
    + fee_whitelist::FeeWhitelistModule
{
    #[init]
    fn init(&self, esdt_safe_address: ManagedAddress, fee: Option<FeeStruct<Self::Api>>) {
        self.init_fee_market(esdt_safe_address, fee);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self) {
        if self.is_setup_phase_complete() {
            return;
        }

        self.setup_phase_complete().set(true);
    }
}
