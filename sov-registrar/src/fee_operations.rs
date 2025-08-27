use structs::fee::FeeStruct;
multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FeeOperationsModule {
    #[endpoint(registerSetFee)]
    fn register_set_fee(&self, fee_struct: FeeStruct<Self::Api>) {
        if let Some(set_fee_error_msg) = self.set_fee_in_storage(&fee_struct) {
            sc_panic!(set_fee_error_msg);
        }
    }
}
