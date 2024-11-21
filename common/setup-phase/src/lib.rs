#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SetupPhaseModule {
    fn require_setup_complete(&self, caller_shard_id: u32) {
        require!(
            self.is_setup_phase_complete(),
            "The setup is not completed in shard {}",
            caller_shard_id
        );
    }

    fn require_caller_initiator(&self) {
        let caller = self.blockchain().get_caller();
        let initiator = self.initiator_address().get();

        require!(caller == initiator, "Invalid caller");
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
