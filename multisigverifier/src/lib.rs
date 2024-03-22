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
        signature: BlsSignature<Self::Api>,
        bridge_operations_hash: ManagedBuffer,
        operations_hashes: MultiValueEncoded<ManagedBuffer>,
    ) {
        let is_bls_valid = self.verify_bls(&signature, &bridge_operations_hash);

        require!(is_bls_valid, "BLS signature is not valid");

        self.calculate_and_check_transfers_hashes(
            &bridge_operations_hash,
            operations_hashes.clone(),
        );

        for operation_hash in operations_hashes {
            self.pending_hashes(bridge_operations_hash.clone())
                .insert(operation_hash);
        }
    }

    #[only_owner]
    #[endpoint(setEsdtSafeAddress)]
    fn set_esdt_safe_address(&self, esdt_safe_address: ManagedAddress) {
        self.esdt_safe_address().set(esdt_safe_address);
    }

    #[endpoint(removeExecutedHash)]
    fn remove_executed_hash(&self, hash_of_hashes: &ManagedBuffer, operation_hash: &ManagedBuffer) {
        let caller = self.blockchain().get_caller();

        require!(
            caller == self.esdt_safe_address().get(),
            "Only ESDT Safe contract can call this endpoint"
        );

        require!(
            self.pending_hashes(hash_of_hashes.clone()).is_empty(),
            "The OutGoingTxsHash has already been registered"
        );

        self.pending_hashes(hash_of_hashes.clone())
            .swap_remove(operation_hash);
    }

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

        require!(
            transfers_hash.eq(hash_of_hashes),
            "Hash of all operations doesn't match the hash of transfer data"
        );
    }

    fn verify_bls(
        &self,
        _signature: &BlsSignature<Self::Api>,
        _bridge_operations_hash: &ManagedBuffer,
    ) -> bool {
        true
    }

    fn is_signature_count_valid(&self, pub_keys_count: usize) -> bool {
        let total_bls_pub_keys = self.bls_pub_keys().len();
        let minimum_signatures = 2 * total_bls_pub_keys / 3;

        pub_keys_count > minimum_signatures
    }

    #[storage_mapper("bls_pub_keys")]
    fn bls_pub_keys(&self) -> SetMapper<ManagedBuffer>;

    #[storage_mapper("pending_hashes")]
    fn pending_hashes(&self, hash_of_hashes: ManagedBuffer) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("esdtSafeAddress")]
    fn esdt_safe_address(&self) -> SingleValueMapper<ManagedAddress>;
}
