#![no_std]

use structs::{
    aliases::EventPaymentTuple,
    configs::{
        UpdateEsdtSafeConfigOperation, UpdateRegistrationStatusOperation,
        UpdateSovereignConfigOperation,
    },
    fee::{
        AddUsersToWhitelistOperation, DistributeFeesOperation, RemoveFeeOperation,
        RemoveUsersFromWhitelistOperation, SetFeeOperation,
    },
    operation::OperationData,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait CustomEventsModule {
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
        error_message: Option<ManagedBuffer>,
    );

    #[event("register")]
    fn register_event(
        &self,
        #[indexed] id: &BigUint,
        #[indexed] address: &ManagedAddress,
        #[indexed] bls_key: &ManagedBuffer,
        #[indexed] egld_stake: &BigUint,
        #[indexed] token_stake: &Option<ManagedVec<EsdtTokenPayment<Self::Api>>>,
    );

    #[event("unregister")]
    fn unregister_event(
        &self,
        #[indexed] id: &BigUint,
        #[indexed] address: &ManagedAddress,
        #[indexed] bls_key: &ManagedBuffer,
        #[indexed] egld_stake: &BigUint,
        #[indexed] token_stake: &Option<ManagedVec<EsdtTokenPayment<Self::Api>>>,
    );

    #[event("completeGenesisPhase")]
    fn complete_genesis_event(&self);

    #[event("registrationStatusUpdate")]
    fn registration_status_update_event(&self, registration_status: &ManagedBuffer);

    #[event("setFee")]
    fn set_fee_event(&self, operation: SetFeeOperation<Self::Api>);

    #[event("removeFee")]
    fn remove_fee_event(&self, operation: RemoveFeeOperation<Self::Api>);

    #[event("distributeFees")]
    fn distribute_fees_event(&self, operation: DistributeFeesOperation<Self::Api>);

    #[event("updateSovereignConfig")]
    fn update_sovereign_config_event(&self, operation: UpdateSovereignConfigOperation<Self::Api>);

    #[event("updateRegistrationStatus")]
    fn update_registration_status_event(&self, operation: UpdateRegistrationStatusOperation);

    #[event("updateEsdtSafeConfig")]
    fn update_esdt_safe_config_event(&self, operation: UpdateEsdtSafeConfigOperation<Self::Api>);

    #[event("addUsersToFeeWhitelist")]
    fn add_users_to_fee_whitelist_event(&self, operation: AddUsersToWhitelistOperation<Self::Api>);

    #[event("removeUsersFromFeeWhitelist")]
    fn remove_users_from_fee_whitelist_event(
        &self,
        operation: RemoveUsersFromWhitelistOperation<Self::Api>,
    );
}
