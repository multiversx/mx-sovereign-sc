use transaction::GasLimit;

use crate::fee_type::FeeType;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct FinalPayment<M: ManagedTypeApi> {
    pub fee: EsdtTokenPayment<M>,
    pub remaining_tokens: EsdtTokenPayment<M>,
}

#[multiversx_sc::module]
pub trait SubtractFeeModule:
    crate::enable_fee::EnableFeeModule
    + crate::fee_type::FeeTypeModule
    + crate::fee_common::CommonFeeModule
    + crate::pairs::PairsModule
    + utils::UtilsModule
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

        let payment = self.call_value().single_esdt();

        if !self.is_fee_enabled() || self.users_whitelist().contains(&original_caller) {
            return FinalPayment {
                fee: EsdtTokenPayment::new(payment.token_identifier.clone(), 0, BigUint::zero()),
                remaining_tokens: payment,
            };
        }

        // TODO: Save fee in storage

        self.subtract_fee_by_type(payment, total_transfers, opt_gas_limit)
    }

    fn subtract_fee_by_type(
        &self,
        mut payment: EsdtTokenPayment,
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
                require!(
                    payment.token_identifier == token,
                    "Invalid token provided for fee"
                );

                let mut total_fee = per_transfer * total_transfers as u32;
                if let OptionalValue::Some(gas_limit) = opt_gas_limit {
                    total_fee += per_gas * gas_limit;
                }

                require!(total_fee <= payment.amount, "Payment does not cover fee");

                payment.amount -= &total_fee;

                FinalPayment {
                    fee: EsdtTokenPayment::new(payment.token_identifier.clone(), 0, total_fee),
                    remaining_tokens: payment,
                }
            }
            FeeType::AnyToken {
                base_fee_token,
                per_transfer,
                per_gas,
            } => {
                todo!();
            }
        }
    }

    #[view(getUsersWhitelist)]
    #[storage_mapper("usersWhitelist")]
    fn users_whitelist(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("accFees")]
    fn accumulated_fees(&self, token_id: TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("tokensForFees")]
    fn tokens_for_fees(&self) -> UnorderedSetMapper<TokenIdentifier>;
}
