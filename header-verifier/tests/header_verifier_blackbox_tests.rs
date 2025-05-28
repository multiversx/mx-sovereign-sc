use common_test_setup::constants::{
    CHAIN_CONFIG_ADDRESS, ENSHRINE_SC_ADDRESS, ESDT_SAFE_ADDRESS, HEADER_VERIFIER_ADDRESS,
};
use error_messages::{
    CURRENT_OPERATION_NOT_REGISTERED, OUTGOING_TX_HASH_ALREADY_REGISTERED,
    SETUP_PHASE_NOT_COMPLETED,
};
use header_verifier::{Headerverifier, OperationHashStatus};
use header_verifier_blackbox_setup::*;
use multiversx_sc::{imports::OptionalValue, types::ManagedBuffer};
use multiversx_sc_scenario::{DebugApi, ScenarioTxWhitebox};
use structs::forge::{ContractInfo, ScArray};

mod header_verifier_blackbox_setup;

#[test]
fn test_deploy() {
    let mut state = HeaderVerifierTestState::new();

    state.common_setup.deploy_header_verifier(vec![]);
}

/// ### TEST
/// H-VERIFIER_REGISTER_ESDT_OK
///
/// ### ACTION
/// Call 'register_esdt_address()' with a valid esdt safe address
///
/// ### EXPECTED
/// The esdt safe address is registered in the contract storage
// #[test]
// fn test_register_esdt_address() {
//     let mut state = HeaderVerifierTestState::new();

//     state
//         .common_setup
//         .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

//     state
//         .common_setup
//         .world
//         .query()
//         .to(HEADER_VERIFIER_ADDRESS)
//         .whitebox(header_verifier::contract_obj, |sc| {
//             let esdt_address = sc.esdt_safe_address().get();

//             assert_eq!(esdt_address, ENSHRINE_SC_ADDRESS);
//         })
// }

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
    let contracts_array = state
        .common_setup
        .get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state.common_setup.deploy_header_verifier(contracts_array);

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.register_operations(operation.clone(), Some(SETUP_PHASE_NOT_COMPLETED));
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

    let contracts_array = state
        .common_setup
        .get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    state.common_setup.deploy_header_verifier(contracts_array);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

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

// FIXME
/// ### TEST
/// H-VERIFIER_REMOVE_HASH_FAIL
///
/// ### ACTION
/// Call 'remove_executed_hash()' without registering any esdt safe address
///
/// ### EXPECTED
/// Error: NO_ESDT_SAFE_ADDRESS
// #[test]
// fn test_remove_executed_hash_no_esdt_address_registered() {
//     let mut state = HeaderVerifierTestState::new();

//     state
//         .common_setup
//         .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

//     state
//         .common_setup
//         .deploy_chain_config(OptionalValue::None, None);

//     state
//         .common_setup
//         .complete_header_verifier_setup_phase(None);

//     let operation_1 = ManagedBuffer::from("operation_1");
//     let operation_2 = ManagedBuffer::from("operation_2");
//     let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

//     state.register_operations(operation.clone(), None);
//     state.remove_executed_hash(
//         ENSHRINE_SC_ADDRESS,
//         &operation.bridge_operation_hash,
//         &operation_1,
//         Some(NO_ESDT_SAFE_ADDRESS),
//     );
// }

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

    let contracts_array = state
        .common_setup
        .get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    state.common_setup.deploy_header_verifier(contracts_array);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_hash_1 = ManagedBuffer::from("operation_1");
    let operation_hash_2 = ManagedBuffer::from("operation_2");
    let operation =
        state.generate_bridge_operation_struct(vec![&operation_hash_1, &operation_hash_2]);

    state.register_operations(operation.clone(), None);
    state.remove_executed_hash(
        ENSHRINE_SC_ADDRESS,
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

    let contracts_array = state
        .common_setup
        .get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    state.common_setup.deploy_header_verifier(contracts_array);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.register_operations(operation.clone(), None);

    state.remove_executed_hash(
        ENSHRINE_SC_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        None,
    );

    state.remove_executed_hash(
        ENSHRINE_SC_ADDRESS,
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

    let contracts_array = state
        .common_setup
        .get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    state.common_setup.deploy_header_verifier(contracts_array);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.lock_operation_hash(
        ENSHRINE_SC_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        Some(CURRENT_OPERATION_NOT_REGISTERED),
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

    let contracts_array = state
        .common_setup
        .get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    state.common_setup.deploy_header_verifier(contracts_array);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.register_operations(operation.clone(), None);

    state.lock_operation_hash(
        ENSHRINE_SC_ADDRESS,
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

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let contracts_array = state
        .common_setup
        .get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    state.common_setup.deploy_header_verifier(contracts_array);

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

    let contracts_array = state
        .common_setup
        .get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    state.common_setup.deploy_header_verifier(contracts_array);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

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
