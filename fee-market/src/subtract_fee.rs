use transaction::GasLimit;

use crate::{fee_type::FeeType, safe_price_query};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const TOTAL_PERCENTAGE: usize = 10_000;

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct FinalPayment<M: ManagedTypeApi> {
    pub fee: EsdtTokenPayment<M>,
    pub remaining_tokens: EsdtTokenPayment<M>,
}

#[derive(TopEncode, TopDecode, ManagedVecItem)]
pub struct AddressPercentagePair<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub percentage: usize,
}

pub struct SubtractPaymentArguments<M: ManagedTypeApi> {
    pub fee_token: TokenIdentifier<M>,
    pub per_transfer: BigUint<M>,
    pub per_gas: BigUint<M>,
    pub payment: EsdtTokenPayment<M>,
    pub total_transfers: usize,
    pub opt_gas_limit: OptionalValue<GasLimit>,
}

#[multiversx_sc::module]
pub trait SubtractFeeModule:
    crate::enable_fee::EnableFeeModule
    + crate::fee_type::FeeTypeModule
    + crate::fee_common::CommonFeeModule
    + crate::price_aggregator::PriceAggregatorModule
    + utils::UtilsModule
    + safe_price_query::SafePriceQueryModule
    + bls_signature::BlsSignatureModule
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
        address_percentage_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
    ) {
        let percentage_total = BigUint::from(TOTAL_PERCENTAGE);

        let mut percentage_sum = 0u64;
        let mut pairs = ManagedVec::<Self::Api, AddressPercentagePair<Self::Api>>::new();
        for pair in address_percentage_pairs {
            let (address, percentage) = pair.into_tuple();
            pairs.push(AddressPercentagePair {
                address,
                percentage,
            });
            percentage_sum += percentage as u64;
        }
        require!(
            percentage_sum == TOTAL_PERCENTAGE as u64,
            "Invalid percentage sum"
        );

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

                    self.send()
                        .direct_esdt(&pair.address, &token_id, 0, &amount_to_send);
                }
            }

            self.accumulated_fees(&token_id).set(&remaining_fees);
        }

        self.tokens_for_fees().clear();
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
        let payment = self.call_value().single_esdt();

        if !self.is_fee_enabled() || self.users_whitelist().contains(&original_caller) {
            self.send()
                .direct_esdt(&caller, &payment.token_identifier, 0, &payment.amount);

            return FinalPayment {
                fee: EsdtTokenPayment::new(payment.token_identifier.clone(), 0, BigUint::zero()),
                remaining_tokens: payment,
            };
        }

        let final_payment = self.subtract_fee_by_type(payment, total_transfers, opt_gas_limit);
        let _ = self
            .tokens_for_fees()
            .insert(final_payment.fee.token_identifier.clone());
        self.accumulated_fees(&final_payment.fee.token_identifier)
            .update(|amt| *amt += &final_payment.fee.amount);

        self.send()
            .direct_non_zero_esdt_payment(&caller, &final_payment.remaining_tokens);

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
            FeeType::None => sc_panic!("Token not accepted as fee"),
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
            "Invalid token provided for fee"
        );

        let mut total_fee = args.per_transfer * args.total_transfers as u32;
        if let OptionalValue::Some(gas_limit) = args.opt_gas_limit {
            total_fee += args.per_gas * gas_limit;
        }

        let mut payment = args.payment;

        require!(total_fee <= payment.amount, "Payment does not cover fee");

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
        let payment_amount_in_fee_token = self.get_safe_price(
            &args.payment.token_identifier,
            &args.fee_token,
        );

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
