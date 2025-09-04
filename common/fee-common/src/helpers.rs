use error_messages::{
    ERROR_AT_ENCODING, INVALID_FEE, INVALID_FEE_TYPE, INVALID_PERCENTAGE_SUM, INVALID_TOKEN_ID,
    INVALID_TOKEN_PROVIDED_FOR_FEE, PAYMENT_DOES_NOT_COVER_FEE, TOKEN_NOT_ACCEPTED_AS_FEE,
};
use structs::{
    aliases::GasLimit,
    fee::{AddressPercentagePair, FeeStruct, FeeType, FinalPayment, SubtractPaymentArguments},
    generate_hash::GenerateHash,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const TOTAL_PERCENTAGE: usize = 10_000;

#[multiversx_sc::module]
pub trait FeeCommonHelpersModule:
    crate::storage::FeeCommonStorageModule + utils::UtilsModule + custom_events::CustomEventsModule
{
    fn distribute_token_fees(
        &self,
        pairs: &ManagedVec<Self::Api, AddressPercentagePair<Self::Api>>,
    ) {
        let percentage_total = BigUint::from(TOTAL_PERCENTAGE);

        for token_id in self.tokens_for_fees().iter() {
            let accumulated_fees = self.accumulated_fees(&token_id).get();
            if accumulated_fees == 0u32 {
                continue;
            }

            let mut remaining_fees = accumulated_fees.clone();

            for pair in pairs {
                let amount_to_send = self.calculate_fee_amount(
                    &accumulated_fees,
                    pair.percentage,
                    &percentage_total,
                );

                if amount_to_send > 0 {
                    remaining_fees -= &amount_to_send;
                    self.send_fee_payment(&pair.address, &token_id, amount_to_send);
                }
            }

            self.accumulated_fees(&token_id).set(&remaining_fees);
        }
    }

    fn parse_pairs(
        &self,
        address_percentage_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
    ) -> ManagedVec<Self::Api, AddressPercentagePair<Self::Api>> {
        let mut pairs = ManagedVec::<Self::Api, AddressPercentagePair<Self::Api>>::new();

        for pair in address_percentage_pairs {
            let (address, percentage) = pair.into_tuple();
            let pair_struct = AddressPercentagePair {
                address,
                percentage,
            };

            pairs.push(pair_struct);
        }

        pairs
    }

    fn generate_pairs_hash(
        &self,
        pairs: &ManagedVec<Self::Api, AddressPercentagePair<Self::Api>>,
        hash_of_hashes: &ManagedBuffer,
    ) -> Option<ManagedBuffer> {
        let mut aggregated_hashes = ManagedBuffer::new();

        for pair in pairs {
            let pair_hash = pair.generate_hash();
            if pair_hash.is_empty() {
                self.complete_operation(
                    hash_of_hashes,
                    &pair_hash,
                    Some(ManagedBuffer::from(ERROR_AT_ENCODING)),
                );
                return None;
            }
            aggregated_hashes.append(&pair_hash);
        }

        let pairs_hash_bytes = self.crypto().sha256(aggregated_hashes);
        Some(pairs_hash_bytes.as_managed_buffer().clone())
    }

    fn calculate_fee_amount(
        &self,
        total_fees: &BigUint,
        percentage: usize,
        percentage_total: &BigUint,
    ) -> BigUint {
        (total_fees * &BigUint::from(percentage)) / percentage_total
    }

    fn send_fee_payment(
        &self,
        address: &ManagedAddress,
        token_id: &TokenIdentifier,
        amount: BigUint,
    ) {
        self.tx()
            .to(address)
            .payment(EsdtTokenPayment::new(token_id.clone(), 0, amount))
            .transfer();
    }

    fn validate_percentage_sum(
        &self,
        pairs: &ManagedVec<Self::Api, AddressPercentagePair<Self::Api>>,
    ) -> Option<ManagedBuffer> {
        let percentage_sum: u64 = pairs.iter().map(|pair| pair.percentage as u64).sum();

        if percentage_sum != TOTAL_PERCENTAGE as u64 {
            return Some(ManagedBuffer::from(INVALID_PERCENTAGE_SUM));
        }

        None
    }

    fn subtract_fee_by_type(
        &self,
        payment: EsdtTokenPayment,
        total_transfers: usize,
        opt_gas_limit: OptionalValue<GasLimit>,
    ) -> FinalPayment<Self::Api> {
        let fee_type = self
            .token_fee(&EgldOrEsdtTokenIdentifier::esdt(
                payment.token_identifier.clone(),
            ))
            .get();
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

    fn set_fee_in_storage(&self, fee_struct: &FeeStruct<Self::Api>) -> Option<&str> {
        if !self.is_valid_token_id(&fee_struct.base_token) {
            return Some(INVALID_TOKEN_ID);
        }

        match &fee_struct.fee_type {
            FeeType::None => sc_panic!(INVALID_FEE_TYPE),
            FeeType::Fixed {
                token,
                per_transfer: _,
                per_gas: _,
            } => {
                require!(&fee_struct.base_token == token, INVALID_FEE);

                token
            }
        };

        self.fee_enabled().set(true);
        self.token_fee(&fee_struct.base_token)
            .set(fee_struct.fee_type.clone());

        None
    }

    fn init_fee_market(
        &self,
        esdt_safe_address: ManagedAddress,
        fee: Option<FeeStruct<Self::Api>>,
    ) {
        self.require_sc_address(&esdt_safe_address);
        self.esdt_safe_address().set(esdt_safe_address);

        match fee {
            Some(fee_struct) => {
                let _ = self.set_fee_in_storage(&fee_struct);
            }
            _ => self.fee_enabled().set(false),
        }
    }
}
