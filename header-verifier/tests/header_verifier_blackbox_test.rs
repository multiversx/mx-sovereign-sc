use error_messages::{
    CURRENT_OPERATION_NOT_REGISTERED, NO_ESDT_SAFE_ADDRESS, ONLY_ESDT_SAFE_CALLER,
};
use header_verifier::{Headerverifier, OperationHashStatus};
use header_verifier_blackbox_setup::*;
use multiversx_sc::types::ManagedBuffer;
use multiversx_sc_scenario::{DebugApi, ScenarioTxWhitebox};

mod header_verifier_blackbox_setup;

#[test]
fn test_deploy() {
    let mut state = HeaderVerifierTestState::new();

    state.deploy();
}

#[test]
fn test_register_esdt_address() {
    let mut state = HeaderVerifierTestState::new();

    state.deploy();
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let esdt_address = sc.esdt_safe_address().get();

            assert_eq!(esdt_address, ENSHRINE_ADDRESS);
        })
}

#[test]
fn test_register_bridge_operation() {
    let mut state = HeaderVerifierTestState::new();

    state.deploy();

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());

    state
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

#[test]
fn test_remove_executed_hash_caller_not_esdt_address() {
    let mut state = HeaderVerifierTestState::new();

    state.deploy();

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);
    state.propose_remove_executed_hash(
        OWNER_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        Some(ONLY_ESDT_SAFE_CALLER),
    );
}

#[test]
fn test_remove_executed_hash_no_esdt_address_registered() {
    let mut state = HeaderVerifierTestState::new();

    state.deploy();

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());
    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        Some(NO_ESDT_SAFE_ADDRESS),
    );
}

#[test]
fn test_remove_one_executed_hash() {
    let mut state = HeaderVerifierTestState::new();

    state.deploy();

    let operation_hash_1 = ManagedBuffer::from("operation_1");
    let operation_hash_2 = ManagedBuffer::from("operation_2");
    let operation =
        state.generate_bridge_operation_struct(vec![&operation_hash_1, &operation_hash_2]);

    state.propose_register_operations(operation.clone());
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_hash_1,
        None,
    );

    state
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

#[test]
fn test_remove_all_executed_hashes() {
    let mut state = HeaderVerifierTestState::new();

    state.deploy();

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        None,
    );

    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_2,
        None,
    );
    state
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

#[test]
fn test_lock_operation_not_registered() {
    let mut state = HeaderVerifierTestState::new();

    state.deploy();
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_lock_operation_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        Some(CURRENT_OPERATION_NOT_REGISTERED),
    );
}

#[test]
fn test_lock_operation() {
    let mut state = HeaderVerifierTestState::new();

    state.deploy();
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());

    state.propose_lock_operation_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        None,
    );

    state
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

#[test]
fn change_validator_set() {
    let mut state = HeaderVerifierTestState::new();

    state.deploy();

    let operation_hash = ManagedBuffer::from("operation_1");
    let hash_of_hashes = state.get_operation_hash(&operation_hash);

    let change_validator_set_log = state.change_validator_set(&hash_of_hashes, &operation_hash);

    assert!(!change_validator_set_log.data.is_empty());
    assert!(!change_validator_set_log.topics.is_empty());
}
