use error_messages::{ERROR_AT_ENCODING, INVALID_PERCENTAGE_SUM};
use structs::{fee::AddressPercentagePair, generate_hash::GenerateHash};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const TOTAL_PERCENTAGE: usize = 10_000;

#[multiversx_sc::module]
pub trait FeeDistributionModule:
    setup_phase::SetupPhaseModule
    + custom_events::CustomEventsModule
    + utils::UtilsModule
    + crate::storage::FeeStorageModule
{
    /// Percentages have to be between 0 and 10_000, and must all add up to 100% (i.e. 10_000)
    #[only_owner]
    #[endpoint(distributeFees)]
    fn distribute_fees(
        &self,
        hash_of_hashes: ManagedBuffer,
        address_percentage_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
    ) {
        self.require_setup_complete();

        let pairs = match self.parse_and_validate_pairs(address_percentage_pairs, &hash_of_hashes) {
            Some(pairs) => pairs,
            None => return,
        };

        let pairs_hash = self.generate_pairs_hash(&pairs, &hash_of_hashes);
        if pairs_hash.is_none() {
            return;
        }
        let pairs_hash = pairs_hash.unwrap();

        self.lock_operation_hash(&hash_of_hashes, &pairs_hash);

        if !self.validate_percentage_sum(&pairs, &hash_of_hashes, &pairs_hash) {
            return;
        }

        self.distribute_token_fees(&pairs);

        self.tokens_for_fees().clear();
        self.complete_operation(&hash_of_hashes, &pairs_hash, None);
    }

    // Helper methods for better separation of concerns

    fn parse_and_validate_pairs(
        &self,
        address_percentage_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
        hash_of_hashes: &ManagedBuffer,
    ) -> Option<ManagedVec<Self::Api, AddressPercentagePair<Self::Api>>> {
        let mut pairs = ManagedVec::<Self::Api, AddressPercentagePair<Self::Api>>::new();

        for pair in address_percentage_pairs {
            let (address, percentage) = pair.into_tuple();
            let pair_struct = AddressPercentagePair {
                address,
                percentage,
            };

            let pair_hash = pair_struct.generate_hash();
            if pair_hash.is_empty() {
                self.complete_operation(
                    hash_of_hashes,
                    &pair_hash,
                    Some(ManagedBuffer::from(ERROR_AT_ENCODING)),
                );
                return None;
            }

            pairs.push(pair_struct);
        }

        Some(pairs)
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

    fn validate_percentage_sum(
        &self,
        pairs: &ManagedVec<Self::Api, AddressPercentagePair<Self::Api>>,
        hash_of_hashes: &ManagedBuffer,
        pairs_hash: &ManagedBuffer,
    ) -> bool {
        let percentage_sum: u64 = pairs.iter().map(|pair| pair.percentage as u64).sum();

        if percentage_sum != TOTAL_PERCENTAGE as u64 {
            self.complete_operation(
                hash_of_hashes,
                pairs_hash,
                Some(ManagedBuffer::from(INVALID_PERCENTAGE_SUM)),
            );
            return false;
        }

        true
    }

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
}
