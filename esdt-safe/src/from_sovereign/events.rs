multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("executedBridgeOp")]
    fn execute_bridge_operation_event(
        &self,
        #[indexed] hash_of_hashes: ManagedBuffer,
        #[indexed] hash_of_bridge_op: ManagedBuffer,
    );

    // #[event("deposit")]
    // fn deposit_event(
    //     &self,
    //     #[indexed] dest_address: &ManagedAddress,
    //     #[indexed] tokens: &MultiValueEncoded<
    //         MultiValue3<TokenIdentifier, u64, StolenFromFrameworkEsdtTokenData<Self::Api>>,
    //     >,
    //     event_data: DepositEvent<Self::Api>,
    // );
}
