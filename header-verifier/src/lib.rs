#![no_std]

use error_messages::{
    BLS_SIGNATURE_NOT_VALID, CURRENT_OPERATION_ALREADY_IN_EXECUTION,
    CURRENT_OPERATION_NOT_REGISTERED, HASH_OF_HASHES_DOES_NOT_MATCH, NO_ESDT_SAFE_ADDRESS,
    ONLY_ESDT_SAFE_CALLER, OUTGOING_TX_HASH_ALREADY_REGISTERED,
};
use multiversx_sc::codec;
use multiversx_sc::proxy_imports::{TopDecode, TopEncode};

multiversx_sc::imports!();

#[derive(TopEncode, TopDecode, PartialEq)]
pub enum OperationHashStatus {
    NotLocked = 1,
    Locked,
}

#[multiversx_sc::contract]
pub trait Headerverifier: cross_chain::events::EventsModule {
    #[init]
    fn init(&self) {}

    #[only_owner]
    #[endpoint(registerBlsPubKeys)]
    fn register_bls_pub_keys(&self, bls_pub_keys: MultiValueEncoded<ManagedBuffer>) {
        self.bls_pub_keys().clear();
        self.bls_pub_keys().extend(bls_pub_keys);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[endpoint(registerBridgeOps)]
    fn register_bridge_operations(
        &self,
        signature: ManagedBuffer,
        bridge_operations_hash: ManagedBuffer,
        _pub_keys_bitmap: ManagedBuffer,
        _epoch: ManagedBuffer,
        operations_hashes: MultiValueEncoded<ManagedBuffer>,
    ) {
        let mut hash_of_hashes_history_mapper = self.hash_of_hashes_history();

        require!(
            !hash_of_hashes_history_mapper.contains(&bridge_operations_hash),
            OUTGOING_TX_HASH_ALREADY_REGISTERED
        );

        let is_bls_valid = self.verify_bls(&signature, &bridge_operations_hash);
        require!(is_bls_valid, BLS_SIGNATURE_NOT_VALID);

        self.calculate_and_check_transfers_hashes(
            &bridge_operations_hash,
            operations_hashes.clone(),
        );

        for operation_hash in operations_hashes {
            self.operation_hash_status(&bridge_operations_hash, &operation_hash)
                .set(OperationHashStatus::NotLocked);
        }

        hash_of_hashes_history_mapper.insert(bridge_operations_hash);
    }

    #[endpoint(changeValidatorSet)]
    fn change_validator_set(
        &self,
        signature: ManagedBuffer,
        bridge_operations_hash: ManagedBuffer,
        operation_hash: ManagedBuffer,
        _pub_keys_bitmap: ManagedBuffer,
        _epoch: ManagedBuffer,
        _pub_keys_id: MultiValueEncoded<ManagedBuffer>,
    ) {
        let mut hash_of_hashes_history_mapper = self.hash_of_hashes_history();

        require!(
            !hash_of_hashes_history_mapper.contains(&bridge_operations_hash),
            OUTGOING_TX_HASH_ALREADY_REGISTERED
        );

        let is_bls_valid = self.verify_bls(&signature, &bridge_operations_hash);
        require!(is_bls_valid, BLS_SIGNATURE_NOT_VALID);

        let mut operations_hashes = MultiValueEncoded::new();
        operations_hashes.push(operation_hash.clone());
        self.calculate_and_check_transfers_hashes(
            &bridge_operations_hash,
            operations_hashes.clone(),
        );

        // TODO change validators set

        hash_of_hashes_history_mapper.insert(bridge_operations_hash.clone());
        self.execute_bridge_operation_event(&bridge_operations_hash, &operation_hash);
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
            CURRENT_OPERATION_NOT_REGISTERED
        );

        let is_hash_in_execution = operation_hash_status_mapper.get();
        match is_hash_in_execution {
            OperationHashStatus::NotLocked => {
                operation_hash_status_mapper.set(OperationHashStatus::Locked)
            }
            OperationHashStatus::Locked => {
                sc_panic!(CURRENT_OPERATION_ALREADY_IN_EXECUTION)
            }
        }
    }

    fn require_caller_esdt_safe(&self) {
        let esdt_safe_mapper = self.esdt_safe_address();

        require!(!esdt_safe_mapper.is_empty(), NO_ESDT_SAFE_ADDRESS);

        let caller = self.blockchain().get_caller();
        require!(caller == esdt_safe_mapper.get(), ONLY_ESDT_SAFE_CALLER);
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
            HASH_OF_HASHES_DOES_NOT_MATCH
        );
    }

    // TODO
    fn verify_bls(
        &self,
        _signature: &ManagedBuffer,
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
