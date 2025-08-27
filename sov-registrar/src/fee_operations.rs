use structs::fee::{FeeStruct, SetFeeOperation};
multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FeeOperationsModule: tx_nonce::TxNonceModule + custom_events::CustomEventsModule {
    #[endpoint(registerSetFee)]
    fn register_set_fee(&self, fee_struct: FeeStruct<Self::Api>) {
        self.set_fee_event(SetFeeOperation {
            fee_struct,
            nonce: self.get_and_save_next_tx_id(),
        });
    }

    // #[endpoint(registerSetFee)]
    // fn register_set_fee(&self, fee_struct: FeeStruct<Self::Api>) {
    //     self.set_fee_event(SetFeeOperation {
    //         fee_struct,
    //         nonce: self.get_and_save_next_tx_id(),
    //     });
    // }
}
