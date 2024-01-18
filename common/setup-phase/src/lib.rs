#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SetupPhaseModule {
    #[endpoint(endSetupPhase)]
    fn end_setup_phase(&self) {
        self.require_caller_initiator();
        self.require_setup_phase();

        self.setup_phase_complete().set(true);
    }

    fn require_caller_initiator(&self) {
        let caller = self.blockchain().get_caller();
        let initiator_address = self.initiator_address().get();
        require!(caller == initiator_address, "Invalid caller");
    }

    fn require_setup_phase(&self) {
        require!(
            !self.is_setup_phase_complete(),
            "Setup phase complete already"
        );
    }

    fn require_setup_complete(&self) {
        require!(
            self.is_setup_phase_complete(),
            "Setup phase must be completed first"
        );
    }

    #[inline]
    fn is_setup_phase_complete(&self) -> bool {
        self.setup_phase_complete().get()
    }

    #[storage_mapper("initiatorAddress")]
    fn initiator_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("setupPhaseComplete")]
    fn setup_phase_complete(&self) -> SingleValueMapper<bool>;
}
