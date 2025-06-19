#![no_std]

use error_messages::{
    BITMAP_LEN_DOES_NOT_MATCH_BLS_KEY_LEN, BLS_KEY_NOT_REGISTERED,
    CALLER_NOT_FROM_CURRENT_SOVEREIGN, CHAIN_CONFIG_NOT_DEPLOYED,
    COULD_NOT_RETRIEVE_SOVEREIGN_CONFIG, CURRENT_OPERATION_ALREADY_IN_EXECUTION,
    CURRENT_OPERATION_NOT_REGISTERED, HASH_OF_HASHES_DOES_NOT_MATCH, INVALID_VALIDATOR_SET_LENGTH,
    MIN_NUMBER_OF_SIGNATURE_NOT_MET, OUTGOING_TX_HASH_ALREADY_REGISTERED,
    VALIDATORS_ALREADY_REGISTERED_IN_EPOCH,
};
use multiversx_sc::codec;
use multiversx_sc::proxy_imports::{TopDecode, TopEncode};
use structs::configs::SovereignConfig;
use structs::forge::{ContractInfo, ScArray};

multiversx_sc::imports!();

#[derive(TopEncode, TopDecode, PartialEq)]
pub enum OperationHashStatus {
    NotLocked = 1,
    Locked,
}

const EPOCH_RANGE: u64 = 3;

#[multiversx_sc::contract]
pub trait Headerverifier: events::EventsModule + setup_phase::SetupPhaseModule {
    #[init]
    fn init(&self, sovereign_contracts: MultiValueEncoded<ContractInfo<Self::Api>>) {
        self.sovereign_contracts().extend(sovereign_contracts);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[endpoint(registerBridgeOps)]
    fn register_bridge_operations(
        &self,
        signature: ManagedBuffer,
        bridge_operations_hash: ManagedBuffer,
        pub_keys_bitmap: ManagedBuffer,
        epoch: u64,
        operations_hashes: MultiValueEncoded<ManagedBuffer>,
    ) {
        self.require_setup_complete();
        let bls_pub_keys_mapper = self.bls_pub_keys(epoch);
        require!(
            pub_keys_bitmap.len() == bls_pub_keys_mapper.len(),
            BITMAP_LEN_DOES_NOT_MATCH_BLS_KEY_LEN
        );

        let mut hash_of_hashes_history_mapper = self.hash_of_hashes_history();

        require!(
            !hash_of_hashes_history_mapper.contains(&bridge_operations_hash),
            OUTGOING_TX_HASH_ALREADY_REGISTERED
        );

        self.verify_bls(
            &signature,
            &bridge_operations_hash,
            pub_keys_bitmap,
            &ManagedVec::from_iter(bls_pub_keys_mapper.iter()),
        );

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
        pub_keys_bitmap: ManagedBuffer,
        epoch: u64,
        pub_keys_id: MultiValueEncoded<BigUint<Self::Api>>,
    ) {
        self.require_setup_complete();
        require!(
            self.bls_pub_keys(epoch).is_empty(),
            VALIDATORS_ALREADY_REGISTERED_IN_EPOCH
        );
        require!(
            pub_keys_bitmap.len() == pub_keys_id.len(),
            BITMAP_LEN_DOES_NOT_MATCH_BLS_KEY_LEN
        );

        let mut hash_of_hashes_history_mapper = self.hash_of_hashes_history();

        require!(
            !hash_of_hashes_history_mapper.contains(&bridge_operations_hash),
            OUTGOING_TX_HASH_ALREADY_REGISTERED
        );

        let bls_keys_previous_epoch = self.bls_pub_keys(epoch - 1);

        self.verify_bls(
            &signature,
            &bridge_operations_hash,
            pub_keys_bitmap,
            &ManagedVec::from_iter(bls_keys_previous_epoch.iter()),
        );

        let mut operations_hashes = MultiValueEncoded::new();
        operations_hashes.push(operation_hash.clone());

        self.calculate_and_check_transfers_hashes(
            &bridge_operations_hash,
            operations_hashes.clone(),
        );

        if epoch > EPOCH_RANGE && !self.bls_pub_keys(epoch - EPOCH_RANGE).is_empty() {
            self.bls_pub_keys(epoch - EPOCH_RANGE).clear();
        }

        let new_bls_keys = self.get_bls_keys_by_id(pub_keys_id);
        self.bls_pub_keys(epoch).extend(new_bls_keys);

        hash_of_hashes_history_mapper.insert(bridge_operations_hash.clone());
        self.execute_bridge_operation_event(&bridge_operations_hash, &operation_hash);
    }

    #[endpoint(removeExecutedHash)]
    fn remove_executed_hash(&self, hash_of_hashes: &ManagedBuffer, operation_hash: &ManagedBuffer) {
        self.require_caller_is_from_current_sovereign();

        self.operation_hash_status(hash_of_hashes, operation_hash)
            .clear();
    }

    #[endpoint(lockOperationHash)]
    fn lock_operation_hash(&self, hash_of_hashes: ManagedBuffer, operation_hash: ManagedBuffer) {
        self.require_caller_is_from_current_sovereign();

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

    #[only_owner]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self) {
        if self.is_setup_phase_complete() {
            return;
        }

        self.check_validator_range(
            self.bls_pub_keys(self.blockchain().get_block_epoch()).len() as u64
        );

        self.setup_phase_complete().set(true);
    }

    fn check_validator_range(&self, number_of_validators: u64) {
        let sovereign_config = self
            .sovereign_config(
                self.sovereign_contracts()
                    .iter()
                    .find(|sc| sc.id == ScArray::ChainConfig)
                    .unwrap_or_else(|| sc_panic!(COULD_NOT_RETRIEVE_SOVEREIGN_CONFIG))
                    .address,
            )
            .get();

        require!(
            number_of_validators >= sovereign_config.min_validators
                && number_of_validators <= sovereign_config.max_validators,
            INVALID_VALIDATOR_SET_LENGTH
        );
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

    fn get_bls_keys_by_id(
        &self,
        ids: MultiValueEncoded<BigUint<Self::Api>>,
    ) -> ManagedVec<ManagedBuffer> {
        let chain_config_address = self
            .sovereign_contracts()
            .iter()
            .find(|sc| sc.id == ScArray::ChainConfig)
            .unwrap_or_else(|| sc_panic!(CHAIN_CONFIG_NOT_DEPLOYED))
            .address;

        let mut bls_keys = ManagedVec::new();

        for id in ids.into_iter() {
            bls_keys.push(
                self.bls_keys_map(chain_config_address.clone())
                    .get(&id)
                    .unwrap_or_else(|| sc_panic!(BLS_KEY_NOT_REGISTERED)),
            );
        }

        bls_keys
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
                        .get(&BigUint::from(index))
                        .unwrap_or_else(|| sc_panic!(BLS_KEY_NOT_REGISTERED)),
                );
            }
        }

        let minimum_signatures = 2 * bls_keys_length / 3 + 1;

        require!(
            approving_validators_bls_keys.len() > minimum_signatures,
            MIN_NUMBER_OF_SIGNATURE_NOT_MET
        );

        approving_validators_bls_keys
    }

    fn get_chain_config_address(&self) -> ManagedAddress {
        self.sovereign_contracts()
            .iter()
            .find(|sc| sc.id == ScArray::ChainConfig)
            .unwrap_or_else(|| sc_panic!(CHAIN_CONFIG_NOT_DEPLOYED))
            .address
    }

    #[storage_mapper("blsPubKeys")]
    fn bls_pub_keys(&self, epoch: u64) -> SetMapper<ManagedBuffer>;

    #[storage_mapper_from_address("blsKeyToId")]
    fn bls_key_to_id_mapper(
        &self,
        sc_address: ManagedAddress,
        bls_key: &ManagedBuffer,
    ) -> SingleValueMapper<BigUint<Self::Api>, ManagedAddress>;

    #[storage_mapper_from_address("blsKeysMap")]
    fn bls_keys_map(
        &self,
        sc_address: ManagedAddress,
    ) -> MapMapper<BigUint<Self::Api>, ManagedBuffer, ManagedAddress>;

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

    #[storage_mapper("chainConfigAddress")]
    fn chain_config_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("sovereignContracts")]
    fn sovereign_contracts(&self) -> UnorderedSetMapper<ContractInfo<Self::Api>>;

    #[storage_mapper_from_address("sovereignConfig")]
    fn sovereign_config(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<SovereignConfig<Self::Api>, ManagedAddress>;
}
