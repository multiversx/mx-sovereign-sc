use error_messages::{
    BLS_KEY_NOT_REGISTERED, CHAIN_CONFIG_NOT_DEPLOYED, HASH_OF_HASHES_DOES_NOT_MATCH,
};
use structs::forge::ScArray;

use crate::checks;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode, PartialEq, Debug)]
pub enum OperationHashStatus {
    NotLocked = 1,
    Locked,
}

pub const MAX_STORED_EPOCHS: u64 = 3;

#[multiversx_sc::module]
pub trait HeaderVerifierUtilsModule:
    super::storage::HeaderVerifierStorageModule
    + checks::HeaderVerifierChecksModule
    + custom_events::CustomEventsModule
    + setup_phase::SetupPhaseModule
    + common_utils::CommonUtilsModule
{
    fn calculate_and_check_transfers_hashes(
        &self,
        transfers_hash: &ManagedBuffer,
        transfers_data: MultiValueEncoded<ManagedBuffer>,
    ) -> Option<ManagedBuffer> {
        let mut transfers_hashes = ManagedBuffer::new();
        for transfer in transfers_data {
            transfers_hashes.append(&transfer);
        }

        let hash_of_hashes_sha256 = self.crypto().sha256(&transfers_hashes);
        let hash_of_hashes = hash_of_hashes_sha256.as_managed_buffer();

        if !self.are_hash_of_hashes_matching(transfers_hash, hash_of_hashes) {
            return Some(HASH_OF_HASHES_DOES_NOT_MATCH.into());
        }

        None
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
        epoch: u64,
        bls_keys_bitmap: &ManagedBuffer,
        bls_keys_length: usize,
    ) -> ManagedVec<ManagedBuffer> {
        let mut padded_bitmap_byte_array = [0u8; 1024];
        let bytes_count = bls_keys_bitmap
            .load_to_byte_array(&mut padded_bitmap_byte_array)
            .len();

        let bitmap_byte_array = &padded_bitmap_byte_array[..bytes_count];

        let mut approving_validators_bls_keys: ManagedVec<Self::Api, ManagedBuffer> =
            ManagedVec::new();

        let bls_keys_from_storage: ManagedVec<ManagedBuffer> =
            self.bls_pub_keys(epoch).iter().collect();

        let mut validator_index: usize = 0;

        'outer: for byte in bitmap_byte_array {
            for bit in 0..8 {
                if validator_index >= bls_keys_length {
                    break 'outer;
                }

                if (byte >> bit) & 1 == 1 {
                    approving_validators_bls_keys
                        .push(bls_keys_from_storage.get(validator_index).clone());
                }

                validator_index += 1;
            }
        }

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

    fn verify_bls(
        &self,
        epoch: u64,
        signature: &ManagedBuffer,
        hash_of_hashes: &ManagedBuffer,
        bls_keys_bitmap: ManagedBuffer,
        bls_pub_keys: &ManagedVec<ManagedBuffer>,
    ) -> Option<ManagedBuffer> {
        let approving_validators =
            self.get_approving_validators(epoch, &bls_keys_bitmap, bls_pub_keys.len());

        self.crypto().verify_bls_aggregated_signature(
            &approving_validators,
            hash_of_hashes,
            signature,
        );

        None
    }

    fn is_caller_from_current_sovereign(&self) -> bool {
        let caller = self.blockchain().get_caller();
        self.sovereign_contracts()
            .iter()
            .any(|sc| sc.address == caller)
    }
}
