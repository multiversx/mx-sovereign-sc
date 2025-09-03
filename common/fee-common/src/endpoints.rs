use structs::{aliases::GasLimit, fee::FinalPayment};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeCommonEndpointsModule:
    crate::helpers::FeeCommonHelpersModule
    + crate::storage::FeeCommonStorageModule
    + common_utils::CommonUtilsModule
    + custom_events::CustomEventsModule
{
    #[payable("*")]
    #[endpoint(subtractFee)]
    fn subtract_fee(
        &self,
        original_caller: ManagedAddress,
        total_transfers: usize,
        opt_gas_limit: OptionalValue<GasLimit>,
    ) -> FinalPayment<Self::Api> {
        self.require_caller_esdt_safe();

        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt().clone();

        if !self.is_fee_enabled() || self.users_whitelist().contains(&original_caller) {
            self.tx().to(&caller).payment(payment.clone()).transfer();

            return FinalPayment {
                fee: EsdtTokenPayment::new(payment.token_identifier.clone(), 0, BigUint::zero()),
                remaining_tokens: payment,
            };
        }

        let final_payment = self.subtract_fee_by_type(payment, total_transfers, opt_gas_limit);

        self.tokens_for_fees()
            .insert(final_payment.fee.token_identifier.clone());

        self.accumulated_fees(&final_payment.fee.token_identifier)
            .update(|amt| *amt += &final_payment.fee.amount);

        if final_payment.remaining_tokens.amount > 0 {
            self.tx()
                .to(&original_caller)
                .payment(&final_payment.remaining_tokens)
                .transfer();
        }

        final_payment
    }
}
