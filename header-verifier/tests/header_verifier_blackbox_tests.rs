use common_test_setup::constants::{
    CHAIN_CONFIG_ADDRESS, ENSHRINE_SC_ADDRESS, HEADER_VERIFIER_ADDRESS,
};
use error_messages::{
    CALLER_NOT_FROM_CURRENT_SOVEREIGN, CURRENT_OPERATION_ALREADY_IN_EXECUTION,
    CURRENT_OPERATION_NOT_REGISTERED, OUTGOING_TX_HASH_ALREADY_REGISTERED,
    SETUP_PHASE_NOT_COMPLETED,
};
use header_verifier::{storage::HeaderVerifierStorageModule, utils::OperationHashStatus};
use header_verifier_blackbox_setup::*;
use multiversx_sc::{
    imports::OptionalValue,
    types::{BigUint, ManagedBuffer, MultiEgldOrEsdtPayment, MultiValueEncoded},
};
use multiversx_sc_scenario::{DebugApi, ScenarioTxWhitebox};
use structs::{configs::SovereignConfig, forge::ScArray};

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

    state.register_operations(
        operation.clone(),
        ManagedBuffer::new(),
        0,
        Some(SETUP_PHASE_NOT_COMPLETED),
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

    let genesis_validator = ManagedBuffer::from("genesis_validator");
    state.common_setup.register_as_validator(
        &genesis_validator,
        &MultiEgldOrEsdtPayment::new(),
        None,
        Some("register"),
    );

    state.common_setup.complete_chain_config_setup_phase(None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let bitmap = ManagedBuffer::new_from_bytes(&[1]);

    state.register_operations(operation.clone(), bitmap.clone(), 0, None);

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

    let genesis_validator = ManagedBuffer::from("genesis_validator");
    state.common_setup.register_as_validator(
        &genesis_validator,
        &MultiEgldOrEsdtPayment::new(),
        None,
        Some("register"),
    );

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let bitmap = ManagedBuffer::new_from_bytes(&[1]);

    state.register_operations(operation.clone(), bitmap, 0, None);
    state.remove_executed_hash(
        ENSHRINE_SC_ADDRESS,
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

    let genesis_validator = ManagedBuffer::from("genesis_validator");
    state.common_setup.register_as_validator(
        &genesis_validator,
        &MultiEgldOrEsdtPayment::new(),
        None,
        Some("register"),
    );

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_hash_1 = ManagedBuffer::from("operation_1");
    let operation_hash_2 = ManagedBuffer::from("operation_2");
    let operation =
        state.generate_bridge_operation_struct(vec![&operation_hash_1, &operation_hash_2]);
    let bitmap = ManagedBuffer::new_from_bytes(&[1]);

    state.register_operations(operation.clone(), bitmap, 0, None);
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

    let genesis_validator = ManagedBuffer::from("genesis_validator");
    state.common_setup.register_as_validator(
        &genesis_validator,
        &MultiEgldOrEsdtPayment::new(),
        None,
        Some("register"),
    );

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let bitmap = ManagedBuffer::new_from_bytes(&[1]);

    state.register_operations(operation.clone(), bitmap, 0, None);

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
        ENSHRINE_SC_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
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

    let genesis_validator = ManagedBuffer::from("genesis_validator");
    state.common_setup.register_as_validator(
        &genesis_validator,
        &MultiEgldOrEsdtPayment::new(),
        None,
        Some("register"),
    );

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let bitmap = ManagedBuffer::new_from_bytes(&[1]);

    state.register_operations(operation.clone(), bitmap, 0, None);

    state.lock_operation_hash(
        CHAIN_CONFIG_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
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

    let genesis_validator = ManagedBuffer::from("genesis_validator");
    state.common_setup.register_as_validator(
        &genesis_validator,
        &MultiEgldOrEsdtPayment::new(),
        None,
        Some("register"),
    );

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let bitmap = ManagedBuffer::new_from_bytes(&[1]);

    state.register_operations(operation.clone(), bitmap, 0, None);

    state.lock_operation_hash(
        CHAIN_CONFIG_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
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
            let is_hash_1_locked = sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_1)
                .get();
            let is_hash_2_locked = sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_2)
                .get();

            assert!(is_hash_1_locked == OperationHashStatus::Locked);
            assert!(is_hash_2_locked == OperationHashStatus::NotLocked);
        });

    state.lock_operation_hash(
        CHAIN_CONFIG_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        Some(CURRENT_OPERATION_ALREADY_IN_EXECUTION),
    );
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

    let genesis_validator = ManagedBuffer::from("genesis_validator");
    state.common_setup.register_as_validator(
        &genesis_validator,
        &MultiEgldOrEsdtPayment::new(),
        None,
        Some("register"),
    );
    state.common_setup.complete_chain_config_setup_phase(None);
    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_hash = ManagedBuffer::from("operation_1");
    let hash_of_hashes = state.get_operation_hash(&operation_hash);

    state.common_setup.update_registration_status(
        &hash_of_hashes,
        1,
        None,
        Some("registrationStatusUpdate"),
    );

    let second_validator = ManagedBuffer::from("second_validator");
    state.common_setup.register_as_validator(
        &second_validator,
        &MultiEgldOrEsdtPayment::new(),
        None,
        Some("register"),
    );
    let third_validator = ManagedBuffer::from("third_validator");
    state.common_setup.register_as_validator(
        &third_validator,
        &MultiEgldOrEsdtPayment::new(),
        None,
        Some("register"),
    );

    let mut validator_set = MultiValueEncoded::new();
    validator_set.push(BigUint::from(1u32));
    validator_set.push(BigUint::from(2u32));
    validator_set.push(BigUint::from(3u32));

    let bitmap = ManagedBuffer::new_from_bytes(&[1]);

    state.change_validator_set(
        &ManagedBuffer::new(),
        &hash_of_hashes,
        &operation_hash,
        1,
        &bitmap,
        validator_set,
        None,
        Some("executedBridgeOp"),
    );
}

/// ### TEST
/// H-VERIFIER_CHANGE_VALIDATORS_FAIL
///
/// ### ACTION
/// Call 'change_validators_set()' after registering the operation
///
/// ### EXPECTED
/// Error OUTGOING_TX_HASH_ALREADY_REGISTERED
#[test]
fn test_change_validator_set_operation_already_registered() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let genesis_validator = ManagedBuffer::from("genesis_validator");
    state.common_setup.register_as_validator(
        &genesis_validator,
        &MultiEgldOrEsdtPayment::new(),
        None,
        Some("register"),
    );

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    let bitmap = ManagedBuffer::new_from_bytes(&[1]);

    state.register_operations(operation.clone(), bitmap.clone(), 0, None);

    state.change_validator_set(
        &ManagedBuffer::new(),
        &operation.bridge_operation_hash,
        &operation.operations_hashes.to_vec().get(0),
        1,
        &bitmap,
        MultiValueEncoded::new(),
        Some(OUTGOING_TX_HASH_ALREADY_REGISTERED),
        None,
    );
}
