use common_test_setup::constants::{
    CHAIN_CONFIG_ADDRESS, ENSHRINE_ADDRESS, HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS,
};
use error_messages::{
    CURRENT_OPERATION_NOT_REGISTERED, NO_ESDT_SAFE_ADDRESS, ONLY_ESDT_SAFE_CALLER,
    OUTGOING_TX_HASH_ALREADY_REGISTERED,
};
use header_verifier::{Headerverifier, OperationHashStatus};
use header_verifier_blackbox_setup::*;
use multiversx_sc::types::ManagedBuffer;
use multiversx_sc_scenario::{DebugApi, ScenarioTxWhitebox};
use structs::configs::SovereignConfig;

mod header_verifier_blackbox_setup;

#[test]
fn test_deploy() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);
}

#[test]
fn test_register_esdt_address() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    state.register_esdt_address(ENSHRINE_ADDRESS);

    state
        .common_setup
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let esdt_address = sc.esdt_safe_address().get();

            assert_eq!(esdt_address, ENSHRINE_ADDRESS);
        })
}

#[test]
fn register_bridge_operation_setup_not_completed() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.register_esdt_address(ENSHRINE_ADDRESS);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.register_operations(operation.clone(), Some("The setup phase must be completed"));
}

#[test]
fn test_register_bridge_operation() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.complete_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.register_operations(operation.clone(), None);

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

#[test]
fn test_remove_executed_hash_caller_not_esdt_address() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.complete_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.register_operations(operation.clone(), None);
    state.register_esdt_address(ENSHRINE_ADDRESS);
    state.remove_executed_hash(
        OWNER_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        Some(ONLY_ESDT_SAFE_CALLER),
    );
}

#[test]
fn test_remove_executed_hash_no_esdt_address_registered() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.complete_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.register_operations(operation.clone(), None);
    state.remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        Some(NO_ESDT_SAFE_ADDRESS),
    );
}

#[test]
fn test_remove_one_executed_hash() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.complete_setup_phase(None);

    let operation_hash_1 = ManagedBuffer::from("operation_1");
    let operation_hash_2 = ManagedBuffer::from("operation_2");
    let operation =
        state.generate_bridge_operation_struct(vec![&operation_hash_1, &operation_hash_2]);

    state.register_esdt_address(ENSHRINE_ADDRESS);

    state.register_operations(operation.clone(), None);
    state.remove_executed_hash(
        ENSHRINE_ADDRESS,
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

#[test]
fn test_remove_all_executed_hashes() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.complete_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.register_esdt_address(ENSHRINE_ADDRESS);

    state.register_operations(operation.clone(), None);

    state.remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        None,
    );

    state.remove_executed_hash(
        ENSHRINE_ADDRESS,
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

#[test]
fn test_lock_operation_not_registered() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    state.register_esdt_address(ENSHRINE_ADDRESS);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.lock_operation_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        Some(CURRENT_OPERATION_NOT_REGISTERED),
    );
}

#[test]
fn test_lock_operation() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.complete_setup_phase(None);

    state.register_esdt_address(ENSHRINE_ADDRESS);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.register_operations(operation.clone(), None);

    state.lock_operation_hash(
        ENSHRINE_ADDRESS,
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

#[test]
fn test_change_validator_set() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    let operation_hash = ManagedBuffer::from("operation_1");
    let hash_of_hashes = state.get_operation_hash(&operation_hash);

    state.change_validator_set(
        &ManagedBuffer::new(),
        &hash_of_hashes,
        &operation_hash,
        None,
        Some("executedBridgeOp"),
    );
}

#[test]
fn test_change_validator_set_operation_already_registered() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.complete_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.register_operations(operation.clone(), None);

    state.change_validator_set(
        &ManagedBuffer::new(),
        &operation.bridge_operation_hash,
        &operation.operations_hashes.to_vec().get(0),
        Some(OUTGOING_TX_HASH_ALREADY_REGISTERED),
        None,
    );
}
