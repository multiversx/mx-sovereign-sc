#![no_std]

use error_messages::ESDT_SAFE_ADDRESS_NOT_SET;
use structs::fee::FeeStruct;

multiversx_sc::imports!();

pub mod fee_distribution;
pub mod fee_type;
pub mod storage;
pub mod subtract_fee;

#[multiversx_sc::contract]
pub trait FeeMarket:
    fee_type::FeeTypeModule
    + subtract_fee::SubtractFeeModule
    + utils::UtilsModule
    + setup_phase::SetupPhaseModule
    + custom_events::CustomEventsModule
    + fee_distribution::FeeDistributionModule
    + storage::FeeStorageModule
{
    #[init]
    fn init(&self, esdt_safe_address: ManagedAddress, fee: Option<FeeStruct<Self::Api>>) {
        self.require_sc_address(&esdt_safe_address);
        self.esdt_safe_address().set(esdt_safe_address);

        match fee {
            Some(fee_struct) => {
                let _ = self.set_fee_in_storage(&fee_struct);
            }
            _ => self.fee_enabled().set(false),
        }
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self) {
        if self.is_setup_phase_complete() {
            return;
        }

        require!(
            !self.esdt_safe_address().is_empty(),
            ESDT_SAFE_ADDRESS_NOT_SET
        );

        self.setup_phase_complete().set(true);
    }
}
