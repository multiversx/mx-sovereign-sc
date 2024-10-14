multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("executedBridgeOp")]
    fn execute_bridge_operation_event(
        &self,
        #[indexed] hash_of_hashes: &ManagedBuffer,
        #[indexed] hash_of_bridge_op: &ManagedBuffer,
    );
}
