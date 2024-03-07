#![no_std]

use bls_signature::BlsSignature;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Multisigverifier: bls_signature::BlsSignatureModule {
    #[init]
    fn init(&self, bls_pub_keys: MultiValueEncoded<ManagedBuffer>) {
        for pub_key in bls_pub_keys {
            self.bls_pub_keys().insert(pub_key);
        }
    }

    #[endpoint]
    fn upgrade(&self) {}

    #[endpoint(registerBridgeOps)]
    fn register_bridge_operations(
        &self,
        bridge_operations_hash: ManagedBuffer,
        operations_hashes: ManagedVec<ManagedBuffer>,
        signature: BlsSignature<Self::Api>,
        // bitmap: MultiValueEncoded<u8>,
    ) {
        let mut bitmap = MultiValueEncoded::new();
        for _ in 0..self.bls_pub_keys().len() {
            bitmap.push(1);
        }

        let is_bls_valid = self.verify_bls(&signature, &bridge_operations_hash, bitmap);

        require!(is_bls_valid, "BLS signature is not valid");

        self.calculate_and_check_transfers_hashes(
            &bridge_operations_hash,
            operations_hashes.clone(),
        );

        for operation in &operations_hashes {
            self.pending_hashes().insert(operation);
        }
    }

    fn calculate_and_check_transfers_hashes(
        &self,
        transfers_hash: &ManagedBuffer,
        transfers_data: ManagedVec<ManagedBuffer>,
    ) {
        let mut transfers_hashes = ManagedBuffer::new();

        for transfer in &transfers_data {
            let transfer_sha256 = self.crypto().sha256(&transfer);
            let transfer_hash = transfer_sha256.as_managed_buffer();

            transfers_hashes.append(transfer_hash);
        }

        let hash_of_hashes_sha256 = self.crypto().sha256(&transfers_hashes);
        let hash_of_hashes = hash_of_hashes_sha256.as_managed_buffer();

        require!(
            transfers_hash.eq(hash_of_hashes),
            "Hash of all operations doesn't match the hash of transfer data"
        );
    }

    fn verify_bls(
        &self,
        _signature: &BlsSignature<Self::Api>,
        _bridge_operations_hash: &ManagedBuffer,
        _bitmap: MultiValueEncoded<u8>,
    ) -> bool {
        // let mut pub_keys: ManagedVec<ManagedBuffer> = ManagedVec::new();
        let is_bls_valid = true;

        // for (pub_key, has_signed) in self.bls_pub_keys().iter().zip(bitmap) {
        //     if has_signed == 1 {
        //         pub_keys.push(pub_key);
        //     }
        // }
        //
        // for pub_key in pub_keys.iter() {
        //     if self.crypto().verify_bls(
        //         &pub_key,
        //         &bridge_operations_hash,
        //         &signature.as_managed_buffer(),
        //     ) == false
        //     {
        //         is_bls_valid = false;
        //         break;
        //     }
        // }

        // if !is_bls_valid {
        //     false
        // } else {
        //     self.is_signature_count_valid(pub_keys.len())
        // }
        is_bls_valid
    }

    fn is_signature_count_valid(&self, pub_keys_count: usize) -> bool {
        let total_bls_pub_keys = self.bls_pub_keys().len();
        let minimum_signatures = 2 * total_bls_pub_keys / 3;

        pub_keys_count > minimum_signatures
    }

    #[storage_mapper("bls_pub_keys")]
    fn bls_pub_keys(&self) -> SetMapper<ManagedBuffer>;

    #[storage_mapper("pending_hashes")]
    fn pending_hashes(&self) -> UnorderedSetMapper<ManagedBuffer>;
}
