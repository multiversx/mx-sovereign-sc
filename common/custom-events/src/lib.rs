#![no_std]

use structs::{
    aliases::{EventPaymentTuple, TxId},
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::{AddressPercentagePair, FeeStruct},
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

    #[event("setFee")]
    fn set_fee_event(&self, #[indexed] fee_struct: FeeStruct<Self::Api>, op_nonce: TxId);

    #[event("removeFee")]
    fn remove_fee_event(
        &self,
        #[indexed] token_id: EgldOrEsdtTokenIdentifier<Self::Api>,
        op_nonce: TxId,
    );

    #[event("distributeFees")]
    fn distribute_fees_event(
        &self,
        #[indexed] operation: ManagedVec<AddressPercentagePair<Self::Api>>,
        op_nonce: TxId,
    );

    #[event("updateSovereignConfig")]
    fn update_sovereign_config_event(
        &self,
        #[indexed] sovereign_config: SovereignConfig<Self::Api>,
        op_nonce: TxId,
    );

    #[event("updateRegistrationStatus")]
    fn update_registration_status_event(&self, #[indexed] registration_status: u8, op_nonce: TxId);

    #[event("updateEsdtSafeConfig")]
    fn update_esdt_safe_config_event(
        &self,
        #[indexed] esdt_safe_config: EsdtSafeConfig<Self::Api>,
        op_nonce: TxId,
    );

    #[event("addUsersToFeeWhitelist")]
    fn add_users_to_fee_whitelist_event(
        &self,
        #[indexed] users: ManagedVec<ManagedAddress<Self::Api>>,
        op_nonce: TxId,
    );

    #[event("removeUsersFromFeeWhitelist")]
    fn remove_users_from_fee_whitelist_event(
        &self,
        #[indexed] users: ManagedVec<ManagedAddress<Self::Api>>,
        op_nonce: TxId,
    );

    #[event("registerToken")]
    fn register_token_event(
        &self,
        #[indexed] token_id: EgldOrEsdtTokenIdentifier<Self::Api>,
        #[indexed] token_type: EsdtTokenType,
        #[indexed] token_name: ManagedBuffer,
        #[indexed] token_ticker: ManagedBuffer,
        #[indexed] token_decimals: usize,
        op_data: OperationData<Self::Api>,
    );
}
