#![no_std]

use structs::{aliases::EventPaymentTuple, operation::OperationData};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("deposit")]
    fn deposit_event(
        &self,
        #[indexed] dest_address: &ManagedAddress,
        #[indexed] tokens: &MultiValueEncoded<EventPaymentTuple<Self::Api>>,
        event_data: OperationData<Self::Api>,
    );

    #[event("scCall")]
    fn sc_call_event(
        &self,
        #[indexed] dest_address: &ManagedAddress,
        event_data: OperationData<Self::Api>,
    );

    #[event("executedBridgeOp")]
    fn execute_bridge_operation_event(
        &self,
        #[indexed] hash_of_hashes: &ManagedBuffer,
        #[indexed] hash_of_bridge_op: &ManagedBuffer,
    );

    #[event("failedBridgeOp")]
    fn failed_bridge_operation_event(
        &self,
        #[indexed] hash_of_hashes: &ManagedBuffer,
        #[indexed] hash: &ManagedBuffer,
        error_message: &ManagedBuffer,
    );

    #[event("register")]
    fn register_event(
        &self,
        #[indexed] id: &BigUint,
        #[indexed] address: &ManagedAddress,
        #[indexed] bls_key: &ManagedBuffer,
        #[indexed] egld_stake: &BigUint,
        #[indexed] token_stake: &Option<ManagedVec<EgldOrEsdtTokenPayment<Self::Api>>>,
    );

    #[event("unregister")]
    fn unregister_event(
        &self,
        #[indexed] id: &BigUint,
        #[indexed] address: &ManagedAddress,
        #[indexed] bls_key: &ManagedBuffer,
        #[indexed] egld_stake: &BigUint,
        #[indexed] token_stake: &ManagedVec<EsdtTokenPayment<Self::Api>>,
    );

    #[event("completeGenesisPhase")]
    fn complete_genesis_event(&self);

    #[event("registrationStatusUpdate")]
    fn registration_status_update_event(&self, registration_status: &ManagedBuffer);
}
