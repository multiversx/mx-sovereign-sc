use structs::fee::{DistributeFeesOperation, FeeStruct, RemoveFeeOperation, SetFeeOperation};
multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FeeOperationsModule:
    tx_nonce::TxNonceModule
    + custom_events::CustomEventsModule
    + fee_common::helpers::FeeCommonHelpersModule
    + fee_common::storage::FeeCommonStorageModule
    + utils::UtilsModule
{
    #[endpoint(registerSetFee)]
    fn register_set_fee(&self, fee_struct: FeeStruct<Self::Api>) {
        self.set_fee_event(SetFeeOperation {
            fee_struct,
            nonce: self.get_and_save_next_tx_id(),
        });
    }

    #[endpoint(registerRemoveFee)]
    fn register_remove_fee(&self, token_id: TokenIdentifier<Self::Api>) {
        self.remove_fee_event(RemoveFeeOperation {
            token_id,
            nonce: self.get_and_save_next_tx_id(),
        });
    }

    #[endpoint(registerDistributeFees)]
    fn register_distribute_fees(
        &self,
        address_percentage_pairs: MultiValueEncoded<MultiValue2<ManagedAddress<Self::Api>, usize>>,
    ) {
        self.distribute_fees_event(DistributeFeesOperation {
            pairs: self.parse_pairs(address_percentage_pairs),
            nonce: self.get_and_save_next_tx_id(),
        });
    }
}
