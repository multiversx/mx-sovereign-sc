#![no_std]

use bls_signature::BlsSignature;
use multiversx_sc::codec;
use multiversx_sc::proxy_imports::{TopDecode, TopEncode};
pub mod header_verifier_proxy;

multiversx_sc::imports!();

#[derive(TopEncode, TopDecode, PartialEq)]
pub enum OperationHashStatus {
    NotLocked = 1,
    Locked,
}

#[multiversx_sc::contract]
pub trait Headerverifier: bls_signature::BlsSignatureModule {
    #[init]
    fn init(&self, bls_pub_keys: MultiValueEncoded<ManagedBuffer>) {
        for pub_key in bls_pub_keys {
            self.bls_pub_keys().insert(pub_key);
        }
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[endpoint(registerBridgeOps)]
    fn register_bridge_operations(
        &self,
        signature: BlsSignature<Self::Api>,
        bridge_operations_hash: ManagedBuffer,
        operations_hashes: MultiValueEncoded<ManagedBuffer>,
    ) {
        require!(
            !self
                .hash_of_hashes_history()
                .contains(&bridge_operations_hash),
            "The OutGoingTxsHash has already been registered"
        );

        let is_bls_valid = self.verify_bls(&signature, &bridge_operations_hash);
        require!(is_bls_valid, "BLS signature is not valid");

        self.calculate_and_check_transfers_hashes(
            &bridge_operations_hash,
            operations_hashes.clone(),
        );

        for operation_hash in operations_hashes {
            self.operation_hash_status(&bridge_operations_hash, &operation_hash)
                .set(OperationHashStatus::NotLocked);
        }

        self.hash_of_hashes_history().insert(bridge_operations_hash);
    }

    #[only_owner]
    #[endpoint(setEsdtSafeAddress)]
    fn set_esdt_safe_address(&self, esdt_safe_address: ManagedAddress) {
        self.esdt_safe_address().set(esdt_safe_address);
    }

    #[endpoint(removeExecutedHash)]
    fn remove_executed_hash(&self, hash_of_hashes: &ManagedBuffer, operation_hash: &ManagedBuffer) {
        self.require_caller_esdt_safe();

        self.operation_hash_status(hash_of_hashes, operation_hash)
            .clear();
    }

    #[endpoint(lockOperationHash)]
    fn lock_operation_hash(&self, hash_of_hashes: ManagedBuffer, operation_hash: ManagedBuffer) {
        self.require_caller_esdt_safe();

        let operation_hash_status_mapper =
            self.operation_hash_status(&hash_of_hashes, &operation_hash);

        require!(
            !operation_hash_status_mapper.is_empty(),
            "The current operation is not registered"
        );

        let is_hash_in_execution = operation_hash_status_mapper.get();
        match is_hash_in_execution {
            OperationHashStatus::NotLocked => {
                operation_hash_status_mapper.set(OperationHashStatus::Locked)
            }
            OperationHashStatus::Locked => {
                sc_panic!("The current operation is already in execution")
            }
        }
    }

    fn require_caller_esdt_safe(&self) {
        let esdt_safe_mapper = self.esdt_safe_address();

        require!(
            !esdt_safe_mapper.is_empty(),
            "There is no registered ESDT address"
        );

        let caller = self.blockchain().get_caller();
        require!(
            caller == esdt_safe_mapper.get(),
            "Only ESDT Safe contract can call this endpoint"
        );
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

    // TODO
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

    #[storage_mapper("blsPubKeys")]
    fn bls_pub_keys(&self) -> SetMapper<ManagedBuffer>;

    #[storage_mapper("operationHashStatus")]
    fn operation_hash_status(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_hash: &ManagedBuffer,
    ) -> SingleValueMapper<OperationHashStatus>;

    #[storage_mapper("hashOfHashesHistory")]
    fn hash_of_hashes_history(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("esdtSafeAddress")]
    fn esdt_safe_address(&self) -> SingleValueMapper<ManagedAddress>;
}
