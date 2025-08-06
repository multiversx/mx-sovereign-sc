use error_messages::{
    BLS_KEY_NOT_REGISTERED, CALLER_NOT_FROM_CURRENT_SOVEREIGN, CHAIN_CONFIG_NOT_DEPLOYED,
};
use structs::forge::ScArray;

use crate::checks;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, PartialEq)]
pub enum OperationHashStatus {
    NotLocked = 1,
    Locked,
}

pub const EPOCH_RANGE: u64 = 3;

#[multiversx_sc::module]
pub trait HeaderVerifierUtilsModule:
    super::storage::HeaderVerifierStorageModule
    + checks::HeaderVerifierChecksModule
    + events::EventsModule
    + setup_phase::SetupPhaseModule
{
    fn calculate_and_check_transfers_hashes(
        &self,
        transfers_hash: &ManagedBuffer,
        transfers_data: MultiValueEncoded<ManagedBuffer>,
    ) {
        let mut transfers_hashes = ManagedBuffer::new();
        for transfer in transfers_data {
            transfers_hashes.append(&transfer);
        }

        let hash_of_hashes_sha256 = self.crypto().sha256(&transfers_hashes);
        let hash_of_hashes = hash_of_hashes_sha256.as_managed_buffer();

        self.require_matching_hash_of_hashes(transfers_hash, hash_of_hashes);
    }

    fn get_chain_config_address(&self) -> ManagedAddress {
        self.sovereign_contracts()
            .iter()
            .find(|sc| sc.id == ScArray::ChainConfig)
            .unwrap_or_else(|| sc_panic!(CHAIN_CONFIG_NOT_DEPLOYED))
            .address
    }

    fn get_approving_validators(
        &self,
        bls_keys_bitmap: &ManagedBuffer,
        bls_keys_length: usize,
    ) -> ManagedVec<ManagedBuffer> {
        let mut padded_bitmap_byte_array = [0u8; 1024];
        bls_keys_bitmap.load_to_byte_array(&mut padded_bitmap_byte_array);

        let bitmap_byte_array = &padded_bitmap_byte_array[..bls_keys_length];

        let mut approving_validators_bls_keys: ManagedVec<Self::Api, ManagedBuffer> =
            ManagedVec::new();

        for (index, has_signed) in bitmap_byte_array.iter().enumerate() {
            if *has_signed == 1u8 {
                approving_validators_bls_keys.push(
                    self.bls_keys_map(self.get_chain_config_address())
                        .get(&BigUint::from(index + 1))
                        .unwrap_or_else(|| sc_panic!(BLS_KEY_NOT_REGISTERED)),
                );
            }
        }

        let minimum_signatures = 2 * bls_keys_length / 3 + 1;

        self.require_min_signatures_amount(approving_validators_bls_keys.len(), minimum_signatures);

        approving_validators_bls_keys
    }

    fn get_bls_keys_by_id(
        &self,
        ids: MultiValueEncoded<BigUint<Self::Api>>,
    ) -> ManagedVec<ManagedBuffer> {
        let mut bls_keys = ManagedVec::new();

        for id in ids.into_iter() {
            bls_keys.push(
                self.bls_keys_map(self.get_chain_config_address())
                    .get(&id)
                    .unwrap_or_else(|| sc_panic!(BLS_KEY_NOT_REGISTERED)),
            );
        }

        bls_keys
    }

    // TODO
    fn verify_bls(
        &self,
        _signature: &ManagedBuffer,
        _bridge_operations_hash: &ManagedBuffer,
        bls_keys_bitmap: ManagedBuffer,
        bls_pub_keys: &ManagedVec<ManagedBuffer>,
    ) {
        let _approving_validators =
            self.get_approving_validators(&bls_keys_bitmap, bls_pub_keys.len());

        // self.crypto().verify_bls_aggregated_signature(
        //     approving_validators,
        //     bridge_operations_hash,
        //     signature,
        // );
    }

    fn require_caller_is_from_current_sovereign(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            self.sovereign_contracts()
                .iter()
                .any(|sc| sc.address == caller),
            CALLER_NOT_FROM_CURRENT_SOVEREIGN
        );
    }
}
