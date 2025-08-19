use error_messages::{
    ERROR_AT_ENCODING, INVALID_PERCENTAGE_SUM, INVALID_TOKEN_PROVIDED_FOR_FEE,
    PAYMENT_DOES_NOT_COVER_FEE, TOKEN_NOT_ACCEPTED_AS_FEE,
};
use structs::{
    aliases::GasLimit,
    fee::{AddressPercentagePair, FeeType, FinalPayment, SubtractPaymentArguments},
    generate_hash::GenerateHash,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const TOTAL_PERCENTAGE: usize = 10_000;

#[multiversx_sc::module]
pub trait SubtractFeeModule:
    crate::fee_type::FeeTypeModule
    + crate::fee_common::CommonFeeModule
    + crate::price_aggregator::PriceAggregatorModule
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

    /// Percentages have to be between 0 and 10_000, and must all add up to 100% (i.e. 10_000)
    #[only_owner]
    #[endpoint(distributeFees)]
    fn distribute_fees(
        &self,
        hash_of_hashes: ManagedBuffer,
        address_percentage_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
    ) {
        self.require_setup_complete();

        let percentage_total = BigUint::from(TOTAL_PERCENTAGE);

        let mut percentage_sum = 0u64;
        let mut pairs = ManagedVec::<Self::Api, AddressPercentagePair<Self::Api>>::new();
        let mut aggregated_hashes = ManagedBuffer::new();

        for pair in address_percentage_pairs {
            let (address, percentage) = pair.into_tuple();
            let pair_struct = AddressPercentagePair {
                address,
                percentage,
            };

            let pair_hash = pair_struct.generate_hash();
            if pair_hash.is_empty() {
                self.failed_bridge_operation_event(
                    &hash_of_hashes,
                    &pair_hash,
                    &ManagedBuffer::from(ERROR_AT_ENCODING),
                );

                self.remove_executed_hash(&hash_of_hashes, &pair_hash);
                return;
            };

            aggregated_hashes.append(&pair_hash);
            pairs.push(pair_struct);

            percentage_sum += percentage as u64;
        }

        let pairs_hash_byte_array = self.crypto().sha256(aggregated_hashes);

        self.lock_operation_hash(&hash_of_hashes, pairs_hash_byte_array.as_managed_buffer());

        if percentage_sum != TOTAL_PERCENTAGE as u64 {
            self.failed_bridge_operation_event(
                &hash_of_hashes,
                pairs_hash_byte_array.as_managed_buffer(),
                &ManagedBuffer::from(INVALID_PERCENTAGE_SUM),
            );

            self.remove_executed_hash(&hash_of_hashes, pairs_hash_byte_array.as_managed_buffer());

            return;
        }

        for token_id in self.tokens_for_fees().iter() {
            let accumulated_fees = self.accumulated_fees(&token_id).get();
            if accumulated_fees == 0u32 {
                continue;
            }

            let mut remaining_fees = accumulated_fees.clone();
            for pair in &pairs {
                let amount_to_send =
                    &(&accumulated_fees * &BigUint::from(pair.percentage)) / &percentage_total;

                if amount_to_send > 0 {
                    remaining_fees -= &amount_to_send;

                    self.tx()
                        .to(&pair.address)
                        .payment(EsdtTokenPayment::new(token_id.clone(), 0, amount_to_send))
                        .transfer();
                }
            }

            self.accumulated_fees(&token_id).set(&remaining_fees);
        }

        self.tokens_for_fees().clear();

        self.remove_executed_hash(&hash_of_hashes, pairs_hash_byte_array.as_managed_buffer());
        self.execute_bridge_operation_event(
            &hash_of_hashes,
            pairs_hash_byte_array.as_managed_buffer(),
        );
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
            FeeType::AnyToken {
                base_fee_token,
                per_transfer,
                per_gas,
            } => {
                let args = SubtractPaymentArguments {
                    fee_token: base_fee_token.clone(),
                    per_transfer,
                    per_gas,
                    payment: payment.clone(),
                    total_transfers,
                    opt_gas_limit,
                };

                if base_fee_token == payment.token_identifier {
                    self.subtract_fee_same_token(args)
                } else {
                    self.subtract_fee_any_token(args)
                }
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

    fn subtract_fee_any_token(
        &self,
        mut args: SubtractPaymentArguments<Self::Api>,
    ) -> FinalPayment<Self::Api> {
        let input_payment = args.payment.clone();
        let payment_amount_in_fee_token =
            self.get_safe_price(&args.payment.token_identifier, &args.fee_token);
        args.payment = EsdtTokenPayment::new(
            args.fee_token.clone(),
            0,
            payment_amount_in_fee_token.clone(),
        );

        let final_payment = self.subtract_fee_same_token(args);
        if final_payment.remaining_tokens.amount == 0 {
            return final_payment;
        }

        // Example: Total cost 1500 RIDE.
        // User pays 100 EGLD, which by asking the pair you found out is 2000 RIDE
        // Then the cost for the user is (1500 RIDE * 100 EGLD / 2000 RIDE = 75 EGLD)
        let fee_amount_in_user_token =
            &final_payment.fee.amount * &input_payment.amount / &payment_amount_in_fee_token;
        let remaining_amount = input_payment.amount - fee_amount_in_user_token;

        FinalPayment {
            fee: final_payment.fee,
            remaining_tokens: EsdtTokenPayment::new(
                input_payment.token_identifier,
                0,
                remaining_amount,
            ),
        }
    }

    #[view(getUsersWhitelist)]
    #[storage_mapper("usersWhitelist")]
    fn users_whitelist(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("accFees")]
    fn accumulated_fees(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("tokensForFees")]
    fn tokens_for_fees(&self) -> UnorderedSetMapper<TokenIdentifier>;
}
