use error_messages::{
    INVALID_TOKEN_PROVIDED_FOR_FEE, PAYMENT_DOES_NOT_COVER_FEE, TOKEN_NOT_ACCEPTED_AS_FEE,
};
use structs::{
    aliases::GasLimit,
    fee::{FeeType, FinalPayment, SubtractPaymentArguments},
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait SubtractFeeModule:
    crate::fee_type::FeeTypeModule
    + crate::storage::FeeStorageModule
    + utils::UtilsModule
    + setup_phase::SetupPhaseModule
    + custom_events::CustomEventsModule
{
    #[only_owner]
    #[endpoint(addUsersToWhitelist)]
    fn add_users_to_whitelist(&self, users: MultiValueEncoded<ManagedAddress>) {
        self.users_whitelist().extend(users);
    }

    #[only_owner]
    #[endpoint(removeUsersFromWhitelist)]
    fn remove_users_from_whitelist(&self, users: MultiValueEncoded<ManagedAddress>) {
        self.remove_items(&mut self.users_whitelist(), users);
    }

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

    fn subtract_fee_by_type(
        &self,
        payment: EsdtTokenPayment,
        total_transfers: usize,
        opt_gas_limit: OptionalValue<GasLimit>,
    ) -> FinalPayment<Self::Api> {
        let fee_type = self.token_fee(&payment.token_identifier).get();
        match fee_type {
            FeeType::None => sc_panic!(TOKEN_NOT_ACCEPTED_AS_FEE),
            FeeType::Fixed {
                token,
                per_transfer,
                per_gas,
            } => {
                let args = SubtractPaymentArguments {
                    fee_token: token,
                    per_transfer,
                    per_gas,
                    payment,
                    total_transfers,
                    opt_gas_limit,
                };
                self.subtract_fee_same_token(args)
            }
        }
    }

    fn subtract_fee_same_token(
        &self,
        args: SubtractPaymentArguments<Self::Api>,
    ) -> FinalPayment<Self::Api> {
        require!(
            args.payment.token_identifier == args.fee_token,
            INVALID_TOKEN_PROVIDED_FOR_FEE
        );

        let mut total_fee = args.per_transfer * args.total_transfers as u32;
        if let OptionalValue::Some(gas_limit) = args.opt_gas_limit {
            total_fee += args.per_gas * gas_limit;
        }

        let mut payment = args.payment;
        require!(total_fee <= payment.amount, PAYMENT_DOES_NOT_COVER_FEE);

        payment.amount -= &total_fee;

        FinalPayment {
            fee: EsdtTokenPayment::new(payment.token_identifier.clone(), 0, total_fee),
            remaining_tokens: payment,
        }
    }
}
