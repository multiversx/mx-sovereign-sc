multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EnableFeeModule {
    #[only_owner]
    #[endpoint(enableFee)]
    fn enable_fee(&self) {
        self.fee_enabled().set(true);
    }

    #[only_owner]
    #[endpoint(disableFee)]
    fn disable_fee(&self) {
        self.fee_enabled().set(false);
    }

    fn is_fee_enabled(&self) -> bool {
        self.fee_enabled().get()
    }

    #[storage_mapper("feeEnabledFlag")]
    fn fee_enabled(&self) -> SingleValueMapper<bool>;
}
