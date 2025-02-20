use operation::{EventPayment, OperationData};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("deposit")]
    fn deposit_event(
        &self,
        #[indexed] dest_address: &ManagedAddress,
        #[indexed] tokens: &MultiValueEncoded<EventPayment<Self::Api>>,
        event_data: OperationData<Self::Api>,
    );

    #[event("executedBridgeOp")]
    fn execute_bridge_operation_event(
        &self,
        #[indexed] hash_of_hashes: &ManagedBuffer,
        #[indexed] hash_of_bridge_op: &ManagedBuffer,
    );
}
