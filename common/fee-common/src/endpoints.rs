use structs::{aliases::GasLimit, fee::FinalPayment};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeCommonEndpointsModule:
    crate::helpers::FeeCommonHelpersModule
    + crate::storage::FeeCommonStorageModule
    + utils::UtilsModule
    + custom_events::CustomEventsModule
{
    fn distribute_fees_common_function(
        &self,
        hash_of_hashes: &ManagedBuffer,
        address_percentage_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
    ) {
        let pairs = match self.parse_and_validate_pairs(address_percentage_pairs, hash_of_hashes) {
            Some(pairs) => pairs,
            None => return,
        };

        let pairs_hash = self.generate_pairs_hash(&pairs, hash_of_hashes);
        if pairs_hash.is_none() {
            return;
        }
        let pairs_hash = pairs_hash.unwrap();

        self.lock_operation_hash(hash_of_hashes, &pairs_hash);

        if !self.validate_percentage_sum(&pairs, hash_of_hashes, &pairs_hash) {
            return;
        }

        self.distribute_token_fees(&pairs);

        self.tokens_for_fees().clear();

        self.complete_operation(hash_of_hashes, &pairs_hash, None);
    }

    fn subtract_fee_common_function(
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
