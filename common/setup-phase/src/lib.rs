#![no_std]

use error_messages::{INVALID_CALLER, SETUP_PHASE_NOT_COMPLETED};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SetupPhaseModule: utils::UtilsModule + custom_events::CustomEventsModule {
    fn require_caller_initiator(&self) {
        let caller = self.blockchain().get_caller();
        let initiator = self.initiator_address().get();

        require!(caller == initiator, INVALID_CALLER);
    }

    #[inline]
    fn require_setup_complete(&self) {
        require!(self.is_setup_phase_complete(), SETUP_PHASE_NOT_COMPLETED);
    }

    fn require_setup_complete_with_event(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_hash: &ManagedBuffer,
    ) {
        self.complete_operation(
            hash_of_hashes,
            operation_hash,
            Some(SETUP_PHASE_NOT_COMPLETED.into()),
        );
    }

    #[inline]
    fn is_setup_phase_complete(&self) -> bool {
        self.setup_phase_complete().get()
    }

    #[storage_mapper("setupPhaseComplete")]
    fn setup_phase_complete(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("initiatorAddress")]
    fn initiator_address(&self) -> SingleValueMapper<ManagedAddress>;
}
