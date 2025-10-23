use common_test_setup::base_setup::helpers::BLSKey;
use common_test_setup::constants::{
    CHAIN_CONFIG_ADDRESS, ESDT_SAFE_ADDRESS, EXECUTED_BRIDGE_OP_EVENT, HEADER_VERIFIER_ADDRESS,
    OWNER_ADDRESS,
};
use error_messages::{
    BLS_KEY_NOT_REGISTERED, CALLER_NOT_FROM_CURRENT_SOVEREIGN,
    CHAIN_CONFIG_SETUP_PHASE_NOT_COMPLETE, CURRENT_OPERATION_ALREADY_IN_EXECUTION,
    CURRENT_OPERATION_NOT_REGISTERED, INCORRECT_OPERATION_NONCE, INVALID_EPOCH,
    NO_VALIDATORS_FOR_GIVEN_EPOCH, NO_VALIDATORS_FOR_PREVIOUS_EPOCH,
    OUTGOING_TX_HASH_ALREADY_REGISTERED, SETUP_PHASE_NOT_COMPLETED,
};
use header_verifier::header_utils::HeaderVerifierUtilsModule;
use header_verifier::storage::HeaderVerifierStorageModule;
use header_verifier_blackbox_setup::*;
use multiversx_sc::imports::{BigUint, ManagedVec, StorageClearable};
use multiversx_sc::types::ReturnsHandledOrError;
use multiversx_sc::{
    imports::OptionalValue,
    types::{ManagedBuffer, MultiEgldOrEsdtPayment, MultiValueEncoded},
};
use multiversx_sc_scenario::api::StaticApi;
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_scenario::{DebugApi, ScenarioTxRun, ScenarioTxWhitebox};
use proxies::header_verifier_proxy::HeaderverifierProxy;
use structs::configs::SovereignConfig;
use structs::OperationHashStatus;
use structs::{forge::ScArray, ValidatorData};

mod header_verifier_blackbox_setup;

#[test]
fn test_deploy() {
    let mut state = HeaderVerifierTestState::new();

    state.common_setup.deploy_header_verifier(vec![]);
}

/// ### TEST
/// H-VERIFIER_REGISTER_OPERATION_FAIL
///
/// ### ACTION
/// Call 'register_operations' with valid operations
///
/// ### EXPECTED
/// Error SETUP_PHASE_NOT_COMPLETED
#[test]
fn register_bridge_operation_setup_not_completed() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let bitmap = state.common_setup.full_bitmap(1);

    state.register_operations(
        &operation.signature,
        operation.clone(),
        bitmap,
        0,
        Some(SETUP_PHASE_NOT_COMPLETED),
    );
}

/// ### TEST
/// H-VERIFIER_REGISTER_OPERATION_NO_VALIDATORS
///
/// ### ACTION
/// Call 'register_operations' without registering validators for the given epoch
///
/// ### EXPECTED
/// Error NO_VALIDATORS_FOR_GIVEN_EPOCH
#[test]
fn test_register_bridge_operation_no_validators_for_epoch() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .register(&BLSKey::random(), &MultiEgldOrEsdtPayment::new(), None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1]);
    let bitmap = state.common_setup.full_bitmap(1);

    let (signature, _pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(1, &operation.bridge_operation_hash);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.register_operations(
        &signature,
        operation,
        bitmap,
        1,
        Some(NO_VALIDATORS_FOR_GIVEN_EPOCH),
    );
}

/// ### TEST
/// H-VERIFIER_REGISTER_OPERATION_OK
///
/// ### ACTION
/// Call 'register_operations' with valid operations and setup completed
///
/// ### EXPECTED
/// The operations are registered in the contract storage
#[test]
fn test_register_bridge_operation() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);
    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let bitmap = state.common_setup.full_bitmap(1);

    let (signature, pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(1, &operation.bridge_operation_hash);

    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.register_operations(&signature, operation.clone(), bitmap.clone(), 0, None);

    state
        .common_setup
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());

            assert!(!sc.hash_of_hashes_history().is_empty());
            assert!(sc.hash_of_hashes_history().len() == 1);
            assert!(sc.hash_of_hashes_history().contains(&hash_of_hashes));

            for operation_hash in operation.operations_hashes {
                let operation_hash_debug_api = ManagedBuffer::from(operation_hash.to_vec());

                let pending_hashes_mapper =
                    sc.operation_hash_status(&hash_of_hashes, &operation_hash_debug_api);

                let is_mapper_empty = pending_hashes_mapper.is_empty();
                let is_operation_hash_locked = pending_hashes_mapper.get();

                assert!(!is_mapper_empty);
                assert!(is_operation_hash_locked == OperationHashStatus::NotLocked);
            }
        });
}

/// ### TEST
/// H-VERIFIER_REMOVE_HASH_FAIL
///
/// ### ACTION
/// Call 'remove_executed_hash()' without registering any esdt safe address
///
/// ### EXPECTED
/// Error: CALLER_NOT_FROM_CURRENT_SOVEREIGN
#[test]
fn test_remove_executed_hash_no_esdt_address_registered() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let bitmap = state.common_setup.full_bitmap(1);

    let (signature, pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(1, &operation.bridge_operation_hash);

    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.register_operations(&signature, operation.clone(), bitmap, 0, None);
    state.remove_executed_hash(
        ESDT_SAFE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        Some(CALLER_NOT_FROM_CURRENT_SOVEREIGN),
    );
}

/// ### TEST
/// H-VERIFIER_REMOVE_HASH_OK
///
/// ### ACTION
/// Call 'remove_executed_hash()' after registering the esdt safe address
///
/// ### EXPECTED
/// The operation hash is removed from the contract storage
#[test]
fn test_remove_one_executed_hash() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let operation_hash_1 = ManagedBuffer::from("operation_1");
    let operation_hash_2 = ManagedBuffer::from("operation_2");
    let operation =
        state.generate_bridge_operation_struct(vec![&operation_hash_1, &operation_hash_2]);
    let bitmap = state.common_setup.full_bitmap(1);

    let (signature, pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(1, &operation.bridge_operation_hash);

    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.register_operations(&signature, operation.clone(), bitmap, 0, None);
    state.remove_executed_hash(
        CHAIN_CONFIG_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_hash_1,
        None,
    );

    state
        .common_setup
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());
            let operation_hash_debug_api = ManagedBuffer::from(operation_hash_2.to_vec());

            let pending_hashes_mapper =
                sc.operation_hash_status(&hash_of_hashes, &operation_hash_debug_api);

            let is_hash_locked = pending_hashes_mapper.get();
            let is_mapper_empty = pending_hashes_mapper.is_empty();

            assert!(!is_mapper_empty);
            assert!(is_hash_locked == OperationHashStatus::NotLocked);
        });
}

/// ### TEST
/// H-VERIFIER_REMOVE_HASH_OK
///
/// ### ACTION
/// Call 'remove_executed_hash()' after registering the esdt safe address
///
/// ### EXPECTED
/// All the operation hashes are removed from the contract storage
#[test]
fn test_remove_all_executed_hashes() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);
    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let bitmap = state.common_setup.full_bitmap(1);

    let (signature, pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(1, &operation.bridge_operation_hash);

    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.register_operations(&signature, operation.clone(), bitmap, 0, None);

    state.remove_executed_hash(
        CHAIN_CONFIG_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        None,
    );

    state.remove_executed_hash(
        CHAIN_CONFIG_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_2,
        None,
    );
    state
        .common_setup
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());
            let operation_hash_debug_api_1 = ManagedBuffer::from(operation_1.to_vec());
            let operation_hash_debug_api_2 = ManagedBuffer::from(operation_2.to_vec());
            assert!(sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_1)
                .is_empty());
            assert!(sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_2)
                .is_empty());
            assert!(sc.hash_of_hashes_history().contains(&hash_of_hashes));
        });
}

/// ### TEST
/// H-VERIFIER_LOCK_OPERATION_FAIL
///
/// ### ACTION
/// Call 'lock_operation_hash()' without registering the operation
///
/// ### EXPECTED
/// Error: CURRENT_OPERATION_NOT_REGISTERED
#[test]
fn test_lock_operation_not_registered() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.lock_operation_hash(
        CHAIN_CONFIG_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        1,
        Some(CURRENT_OPERATION_NOT_REGISTERED),
    );
}

/// ### TEST
/// H-VERIFIER_LOCK_OPERATION_FAIL
///
/// ### ACTION
/// Call 'lock_operation_hash()' from an unregistered sc
///
/// ### EXPECTED
/// Error: CALLER_NOT_FROM_CURRENT_SOVEREIGN
#[test]
fn test_lock_operation_caller_not_from_sovereign() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.lock_operation_hash(
        ESDT_SAFE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        0,
        Some(CALLER_NOT_FROM_CURRENT_SOVEREIGN),
    );
}

/// ### TEST
/// H-VERIFIER_LOCK_OPERATION_OK
///
/// ### ACTION
/// Call 'lock_operation_hash()' after registering the operations
///
/// ### EXPECTED
/// Only the first operation hash is locked in the contract storage
#[test]
fn test_lock_operation() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let bitmap = state.common_setup.full_bitmap(1);

    let (signature, pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(1, &operation.bridge_operation_hash);

    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.register_operations(&signature, operation.clone(), bitmap, 0, None);

    state.assert_last_operation_nonce(0);

    let expected_operation_nonce = state.next_operation_nonce();

    state.lock_operation_hash(
        CHAIN_CONFIG_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        expected_operation_nonce,
        None,
    );

    state.assert_last_operation_nonce(expected_operation_nonce);

    state
        .common_setup
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());
            let operation_hash_debug_api_1 = ManagedBuffer::from(operation_1.to_vec());
            let operation_hash_debug_api_2 = ManagedBuffer::from(operation_2.to_vec());
            let is_hash_1_locked = sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_1)
                .get();
            let is_hash_2_locked = sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_2)
                .get();

            assert!(is_hash_1_locked == OperationHashStatus::Locked);
            assert!(is_hash_2_locked == OperationHashStatus::NotLocked);
        })
}

/// ### TEST
/// H-VERIFIER_LOCK_OPERATION_FAIL
///
/// ### ACTION
/// Call 'lock_operation_hash()' with a stale operation nonce value
///
/// ### EXPECTED
/// Error: INCORRECT_OPERATION_NONCE
#[test]
fn test_lock_operation_incorrect_nonce_rejected() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let operation_hash_1 = ManagedBuffer::from("operation_nonce_fail_1");
    let operation_hash_2 = ManagedBuffer::from("operation_nonce_fail_2");
    let operation =
        state.generate_bridge_operation_struct(vec![&operation_hash_1, &operation_hash_2]);
    let bitmap = state.common_setup.full_bitmap(1);

    let (signature, pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(1, &operation.bridge_operation_hash);

    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.register_operations(&signature, operation.clone(), bitmap, 0, None);

    state.assert_last_operation_nonce(0);
    let expected_next_nonce = state.next_operation_nonce();
    let incorrect_nonce = expected_next_nonce.checked_add(1).unwrap();

    assert_eq!(
        state
            .common_setup
            .world
            .tx()
            .from(CHAIN_CONFIG_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .lock_operation_hash(
                operation.bridge_operation_hash,
                operation_hash_1,
                incorrect_nonce,
            )
            .returns(ReturnsHandledOrError::new())
            .run()
            .err()
            .unwrap()
            .message,
        INCORRECT_OPERATION_NONCE
    );

    state.assert_last_operation_nonce(expected_next_nonce);
}

/// ### TEST
/// H-VERIFIER_LOCK_OPERATION_FAIL
///
/// ### ACTION
/// Call 'lock_operation_hash()' on already locked hash
///
/// ### EXPECTED
/// Error CURRENT_OPERATION_ALREADY_IN_EXECUTION
#[test]
fn test_lock_operation_hash_already_locked() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let bitmap = state.common_setup.full_bitmap(1);

    let (signature, pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(1, &operation.bridge_operation_hash);

    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);
    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.register_operations(&signature, operation.clone(), bitmap, 0, None);

    state.assert_last_operation_nonce(0);

    let expected_operation_nonce = state.next_operation_nonce();

    state.lock_operation_hash(
        CHAIN_CONFIG_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        expected_operation_nonce,
        None,
    );

    state.assert_last_operation_nonce(expected_operation_nonce);

    state
        .common_setup
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());
            let operation_hash_debug_api_1 = ManagedBuffer::from(operation_1.to_vec());
            let operation_hash_debug_api_2 = ManagedBuffer::from(operation_2.to_vec());
            let is_hash_1_locked = sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_1)
                .get();
            let is_hash_2_locked = sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_2)
                .get();

            assert!(is_hash_1_locked == OperationHashStatus::Locked);
            assert!(is_hash_2_locked == OperationHashStatus::NotLocked);
        });

    let next_operation_nonce = state.next_operation_nonce();

    state.lock_operation_hash(
        CHAIN_CONFIG_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        next_operation_nonce,
        Some(CURRENT_OPERATION_ALREADY_IN_EXECUTION),
    );

    state.assert_last_operation_nonce(expected_operation_nonce);
}

/// ### TEST
/// H-VERIFIER_CHANGE_VALIDATORS_OK
///
/// ### ACTION
/// Call 'change_validators_set()' with a valid operation hash
///
/// ### EXPECTED
/// The validator set is changed in the contract storage
#[test]
fn test_change_validator_set() {
    let mut state = HeaderVerifierTestState::new();
    let sovereign_config = SovereignConfig {
        max_validators: 3,
        ..SovereignConfig::default_config()
    };

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(sovereign_config), None);

    let mut registered_bls_keys: ManagedVec<StaticApi, ManagedBuffer<StaticApi>> =
        ManagedVec::new();
    let operation_hash = ManagedBuffer::from("operation_1");
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    let (signature, pub_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    let genesis_validator = pub_keys[0].clone();
    registered_bls_keys.push(genesis_validator.clone());
    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();
    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    for id in 2..4 {
        let validator_bls_key = BLSKey::random();
        registered_bls_keys.push(validator_bls_key.clone());
        let validator_data = ValidatorData {
            id: BigUint::from(id as u32),
            address: OWNER_ADDRESS.to_managed_address(),
            bls_key: validator_bls_key,
        };

        let bitmap = state.common_setup.full_bitmap(1);
        let epoch = 0;
        state.common_setup.register_validator_operation(
            validator_data,
            signature.clone(),
            bitmap.clone(),
            epoch,
        );
    }

    let mut validator_set = MultiValueEncoded::new();
    validator_set.push(BigUint::from(1u32));
    validator_set.push(BigUint::from(2u32));
    validator_set.push(BigUint::from(3u32));

    let bitmap = state.common_setup.full_bitmap(1);
    let epoch_for_new_set = 1;

    let (change_validator_set_sig, change_validator_set_pub_keys) =
        state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let pub_key = ManagedBuffer::new_from_bytes(&change_validator_set_pub_keys[0].to_vec());
            sc.bls_pub_keys(0).clear();
            sc.bls_pub_keys(0).insert(pub_key);
        });

    state.change_validator_set(
        &change_validator_set_sig,
        &hash_of_hashes,
        &operation_hash,
        epoch_for_new_set,
        &bitmap,
        validator_set,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        None,
    );

    state
        .common_setup
        .check_bls_key_for_epoch_in_header_verifier(epoch_for_new_set, &registered_bls_keys);
}

/// ### TEST
/// H-VERIFIER_CHANGE_VALIDATORS_FAIL
///
/// ### ACTION
/// Call 'change_validator_set()' for the genesis epoch
///
/// ### EXPECTED
/// Error INVALID_EPOCH is emitted
#[test]
fn test_change_validator_invalid_epoch() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let genesis_validator = BLSKey::random();
    state
        .common_setup
        .register(&genesis_validator, &MultiEgldOrEsdtPayment::default(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_hash = ManagedBuffer::from("operation_1");
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
    let (signature, _) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    let bitmap = state.common_setup.full_bitmap(1);
    let validator_set = MultiValueEncoded::new();
    let epoch = 0u64;

    state.change_validator_set(
        &signature,
        &hash_of_hashes,
        &operation_hash,
        epoch,
        &bitmap,
        validator_set,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(INVALID_EPOCH),
    );
}

/// ### TEST
/// H-VERIFIER_CHANGE_VALIDATORS_FAIL
///
/// ### ACTION
/// Call 'change_validator_set()' when the previous epoch has no registered validators
///
/// ### EXPECTED
/// Error NO_VALIDATORS_FOR_PREVIOUS_EPOCH is emitted
#[test]
fn change_validator_set_previous_epoch_has_no_validators() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let genesis_validator = BLSKey::random();
    state
        .common_setup
        .register(&genesis_validator, &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_hash = ManagedBuffer::from("operation_1");
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
    let signature = ManagedBuffer::new();
    let bitmap = ManagedBuffer::new();
    let validator_set = MultiValueEncoded::new();
    let epoch = 2u64;

    state.change_validator_set(
        &signature,
        &hash_of_hashes,
        &operation_hash,
        epoch,
        &bitmap,
        validator_set,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(NO_VALIDATORS_FOR_PREVIOUS_EPOCH),
    );
}

/// ### TEST
/// H-VERIFIER_CHANGE_VALIDATORS_FAIL
///
/// ### ACTION
/// Call 'change_validator_set()' before registering the operation
///
/// ### EXPECTED
/// Error OUTGOING_TX_HASH_ALREADY_REGISTERED
#[test]
fn test_change_validator_set_operation_already_registered() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let operation_hash_1 = ManagedBuffer::from("operation_1");
    let operation = state.generate_bridge_operation_struct(vec![&operation_hash_1]);
    let bitmap = state.common_setup.full_bitmap(1);

    let (signature, pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(1, &operation.bridge_operation_hash);

    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.change_validator_set(
        &signature,
        &operation.bridge_operation_hash,
        &operation_hash_1,
        1,
        &bitmap,
        MultiValueEncoded::new(),
        Some(EXECUTED_BRIDGE_OP_EVENT),
        None,
    );

    state.change_validator_set(
        &signature,
        &operation.bridge_operation_hash,
        &operation_hash_1,
        1,
        &bitmap,
        MultiValueEncoded::new(),
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(OUTGOING_TX_HASH_ALREADY_REGISTERED),
    );
}

/// ### TEST
/// H-VERIFIER_CHANGE_VALIDATORS_FAIL_BLS_KEY
///
/// ### ACTION
/// Call 'change_validator_set()' with a validator id that is not registered
///
/// ### EXPECTED
/// Error BLS_KEY_NOT_REGISTERED is emitted
#[test]
fn test_change_validator_set_bls_key_not_found() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let operation_hash = ManagedBuffer::from("operation_missing_validator");
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    let (signature, pub_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            sc.bls_pub_keys(0).clear();
            sc.bls_pub_keys(0)
                .insert(ManagedBuffer::new_from_bytes(&pub_keys[0].to_vec()));
        });

    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 1u64;

    let mut validator_set = MultiValueEncoded::new();
    validator_set.push(BigUint::from(999u32));

    state.change_validator_set(
        &signature,
        &hash_of_hashes,
        &operation_hash,
        epoch,
        &bitmap,
        validator_set,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(BLS_KEY_NOT_REGISTERED),
    );
}

/// ### TEST
/// H-VERIFIER_CHANGE_VALIDATORS_OK
///
/// ### ACTION
/// Call 'change_validators_set()' for four epochs
///
/// ### EXPECTED
/// The validator set is changed in the contract storage and the genesis epoch is cleared
#[ignore = "Ignore until workaround is found"]
#[test]
fn test_change_multiple_validator_sets() {
    let mut state = HeaderVerifierTestState::new();
    let sovereign_config = SovereignConfig {
        max_validators: 11,
        ..SovereignConfig::default_config()
    };

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(sovereign_config), None);

    state
        .common_setup
        .register(&BLSKey::random(), &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let mut last_bls_key_id = 1u32;
    for epoch in 1..10 {
        let validator_bls_key = BLSKey::random();
        last_bls_key_id += 1;
        let validator_data = ValidatorData {
            id: BigUint::from(last_bls_key_id),
            address: OWNER_ADDRESS.to_managed_address(),
            bls_key: validator_bls_key.clone(),
        };

        let signature = ManagedBuffer::new();

        let bitmap = state.common_setup.full_bitmap(1);

        state.common_setup.register_validator_operation(
            validator_data,
            signature.clone(),
            bitmap.clone(),
            epoch - 1,
        );

        let operation_hash = ManagedBuffer::from(format!("validators_epoch_{}", epoch));
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
        let mut validator_set = MultiValueEncoded::new();
        validator_set.push(BigUint::from(epoch + 1));

        state.change_validator_set(
            &ManagedBuffer::new(),
            &hash_of_hashes,
            &operation_hash,
            epoch,
            &bitmap,
            validator_set,
            Some(EXECUTED_BRIDGE_OP_EVENT),
            None,
        );

        let mut bls_keys: ManagedVec<StaticApi, ManagedBuffer<StaticApi>> = ManagedVec::new();
        bls_keys.push(validator_bls_key);
        state
            .common_setup
            .check_bls_key_for_epoch_in_header_verifier(epoch, &bls_keys);

        if epoch >= 3 {
            state
                .common_setup
                .world
                .query()
                .to(HEADER_VERIFIER_ADDRESS)
                .whitebox(header_verifier::contract_obj, |sc| {
                    assert!(sc.bls_pub_keys(epoch - 3).is_empty());
                })
        }
    }
}

/// ### TEST
/// H-VERIFIER_COMPLETE_SETUP_PHASE
///
/// ### ACTION
/// Call 'complete_setup_phase()' without chain config setup completed
///
/// ### EXPECTED
/// Error: CHAIN_CONFIG_SETUP_PHASE_NOT_COMPLETE
#[test]
fn test_complete_setup_phase_chain_config_fail() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(Some(CHAIN_CONFIG_SETUP_PHASE_NOT_COMPLETE));
}

#[test]
fn test_get_approving_validators() {
    let mut state = HeaderVerifierTestState::new();

    state.common_setup.deploy_header_verifier(vec![]);

    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            // Create hardcoded BLS keys for testing
            let validator0_bls_key = BLSKey::random(); // genesis validator
            let validator1_bls_key = BLSKey::random();
            let validator2_bls_key = BLSKey::random();
            let validator3_bls_key = BLSKey::random();
            let validator4_bls_key = BLSKey::random();

            let epoch = 0u64;

            // Store BLS keys in the contract
            sc.bls_pub_keys(epoch).insert(validator0_bls_key.clone());
            sc.bls_pub_keys(epoch).insert(validator1_bls_key.clone());
            sc.bls_pub_keys(epoch).insert(validator2_bls_key.clone());
            sc.bls_pub_keys(epoch).insert(validator3_bls_key.clone());
            sc.bls_pub_keys(epoch).insert(validator4_bls_key.clone());

            // Test Case 1: Bitmap [0b00000001] - Only validator at index 0 approves
            let bitmap = ManagedBuffer::new_from_bytes(&[0b00000001]);
            let approving_validators = sc.get_approving_validators(epoch, &bitmap, 5);
            assert_eq!(approving_validators.len(), 1);
            assert_eq!(approving_validators.get(0).clone(), validator0_bls_key);

            // Test Case 2: Bitmap [0b00000101] - Validators at indices 0 and 2 approve
            let bitmap = ManagedBuffer::new_from_bytes(&[0b00000101]);
            let approving_validators = sc.get_approving_validators(epoch, &bitmap, 5);
            assert_eq!(approving_validators.len(), 2);
            assert_eq!(approving_validators.get(0).clone(), validator0_bls_key);
            assert_eq!(approving_validators.get(1).clone(), validator2_bls_key);

            // Test Case 3: Bitmap [0b11111111] - All validators approve
            let bitmap = ManagedBuffer::new_from_bytes(&[0b11111111]);
            let approving_validators = sc.get_approving_validators(epoch, &bitmap, 5);
            assert_eq!(approving_validators.len(), 5);
            assert_eq!(approving_validators.get(0).clone(), validator0_bls_key);
            assert_eq!(approving_validators.get(1).clone(), validator1_bls_key);
            assert_eq!(approving_validators.get(2).clone(), validator2_bls_key);
            assert_eq!(approving_validators.get(3).clone(), validator3_bls_key);
            assert_eq!(approving_validators.get(4).clone(), validator4_bls_key);

            // Test Case 4: Bitmap [0b00000000] - No validators approve
            let bitmap = ManagedBuffer::new_from_bytes(&[0b00000000]);
            let approving_validators = sc.get_approving_validators(epoch, &bitmap, 5);
            assert_eq!(approving_validators.len(), 0);

            // Test Case 5: Bitmap [0b00001010] - Validators at indices 1 and 3 approve
            let bitmap = ManagedBuffer::new_from_bytes(&[0b00001010]);
            let approving_validators = sc.get_approving_validators(epoch, &bitmap, 5);
            assert_eq!(approving_validators.len(), 2);
            assert_eq!(approving_validators.get(0).clone(), validator1_bls_key);
            assert_eq!(approving_validators.get(1).clone(), validator3_bls_key);
        });
}
