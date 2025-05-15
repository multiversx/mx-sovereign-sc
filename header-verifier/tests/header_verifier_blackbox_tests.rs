use common_test_setup::constants::{
    CHAIN_CONFIG_ADDRESS, ENSHRINE_SC_ADDRESS, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS,
    FIRST_TEST_TOKEN, HEADER_VERIFIER_ADDRESS,
};
use error_messages::{
    CALLER_IS_NOT_OWNER, CURRENT_OPERATION_NOT_REGISTERED, ESDT_SAFE_ADDRESS_NOT_SET,
    OUTGOING_TX_HASH_ALREADY_REGISTERED, SETUP_PHASE_NOT_COMPLETED,
};
use header_verifier::{Headerverifier, OperationHashStatus};
use header_verifier_blackbox_setup::*;
use multiversx_sc::{
    imports::OptionalValue,
    types::{ManagedBuffer, MultiValueEncoded},
};
use multiversx_sc_scenario::{DebugApi, ScenarioTxWhitebox};
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::{FeeStruct, FeeType},
};

mod header_verifier_blackbox_setup;

#[test]
fn test_deploy() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
}

/// Test that registers the ESDT-Safe address
#[test]
fn test_register_esdt_address() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state.register_esdt_address(ENSHRINE_SC_ADDRESS);

    state
        .common_setup
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let esdt_address = sc.esdt_safe_address().get();

            assert_eq!(esdt_address, ENSHRINE_SC_ADDRESS);
        })
}

/// Test that register bridge operation fails because the setup phase was not completed
#[test]
fn register_bridge_operation_setup_not_completed() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.register_esdt_address(ENSHRINE_SC_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.register_operations(operation.clone(), Some(SETUP_PHASE_NOT_COMPLETED));
}

/// Test that successfully registeres a bridge operation
/// Steps:
/// 1. Deploy the Header-Verifier and Chain-Config contracts
/// 2. Complete the setup phase
/// 3. Register the Operation
/// 4. Check inside the contracts storage that the Operation was registered
#[test]
fn test_register_bridge_operation() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.register_esdt_address(ENSHRINE_SC_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

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

/// Test that the removal of an executed hash fails because the caller is not an ESDT-Safe contract
#[test]
fn test_complete_setup_phase_no_esdt_address_registered() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state
        .common_setup
        .complete_header_verifier_setup_phase(Some(ESDT_SAFE_ADDRESS_NOT_SET));
}

/// Test that successfully removes one executed hash from the contract
/// Steps:
/// 1. Deploy the Header-Verifier and Chain-Config contracts
/// 2. Complete the setup phase
/// 3. Register the ESDT-Safe address
/// 4. Register the Operation
/// 5. Remove the executed Operation hash
/// 6. Check in the contracts storage that the hash was removed
#[test]
fn test_remove_one_executed_hash() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.register_esdt_address(ENSHRINE_SC_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

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

/// Test that successfully removes all executed hashes
/// Steps:
/// 1. Deploy the Header-Verifier and Chain-Config contracts
/// 2. Complete the setup phase
/// 3. Register the ESDT-Safe address
/// 4. Register the Operation
/// 5. Remove the executed Operation hashes
/// 6. Check in the contracts storage that the hash was removed
#[test]
fn test_remove_all_executed_hashes() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.register_esdt_address(ENSHRINE_SC_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

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

/// Test that fails the lock of an Operation because it was not registered
#[test]
fn test_lock_operation_not_registered() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state.register_esdt_address(ENSHRINE_SC_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

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

/// Test that successfully lock an Operation hash
/// Steps:
/// 1. Deploy the Header-Verifier and Chain-Config contracts
/// 2. Complete the setup phase
/// 3. Register the ESDT-Safe address
/// 4. Register the Operation
/// 5. Lock the Operation hash
/// 6. Check in the contracts storage that the hash was locked
#[test]
fn test_lock_operation() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.register_esdt_address(ENSHRINE_SC_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

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

/// Test that successfully changes the validator set
/// Steps:
/// 1. Deploy the Header-Verifier contract
/// 2. Change the validator set with log assertion
#[test]
fn test_change_validator_set() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

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

/// Test that fails the change of validator set because the operation was already registered
#[test]
fn test_change_validator_set_operation_already_registered() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.register_esdt_address(ENSHRINE_SC_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

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

#[test]
fn test_update_sovereign_config_sovereign_setup_phase_not_complete() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.register_esdt_address(ENSHRINE_SC_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.update_sovereign_config(
        &SovereignConfig::default_config(),
        Some(CALLER_IS_NOT_OWNER),
    );
}

#[test]
fn test_update_sovereign_config_header_verifier_setup_phase_not_complete() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.register_esdt_address(ENSHRINE_SC_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

    state.update_sovereign_config(
        &SovereignConfig::default_config(),
        Some(SETUP_PHASE_NOT_COMPLETED),
    );
}

#[test]
fn test_update_esdt_safe_config_header_verifier_setup_phase_not_complete() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state.update_esdt_safe_config(
        &EsdtSafeConfig::default_config(),
        Some(SETUP_PHASE_NOT_COMPLETED),
    );
}

#[test]
fn test_update_esdt_safe_config_sovereign_setup_phase_not_complete() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.register_esdt_address(ESDT_SAFE_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.update_esdt_safe_config(&EsdtSafeConfig::default_config(), Some(CALLER_IS_NOT_OWNER));
}

#[test]
fn test_set_fee_sovereign_setup_phase_not_completed() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state.register_esdt_address(ESDT_SAFE_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let fee = FeeStruct {
        base_token: FIRST_TEST_TOKEN.to_token_identifier(),
        fee_type: FeeType::None,
    };

    state.set_fee(fee, Some(CALLER_IS_NOT_OWNER));
}

#[test]
fn test_set_fee_header_verifier_setup_phase_not_completed() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    let fee = FeeStruct {
        base_token: FIRST_TEST_TOKEN.to_token_identifier(),
        fee_type: FeeType::None,
    };

    state.set_fee(fee, Some(SETUP_PHASE_NOT_COMPLETED));
}

#[test]
fn test_remove_fee_sovereign_setup_phase_not_completed() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state.register_esdt_address(ESDT_SAFE_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.remove_fee(FIRST_TEST_TOKEN, Some(CALLER_IS_NOT_OWNER));
}

#[test]
fn test_remove_fee_header_verifier_setup_phase_not_completed() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state.remove_fee(FIRST_TEST_TOKEN, Some(SETUP_PHASE_NOT_COMPLETED));
}

#[test]
fn test_distribute_fee_header_verifier_setup_phase_not_completed() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state.distribute_fee(MultiValueEncoded::new(), Some(SETUP_PHASE_NOT_COMPLETED));
}

#[test]
fn test_distribute_fee_sovereign_verifier_setup_phase_not_completed() {
    let mut state = HeaderVerifierTestState::new();

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state.register_esdt_address(ESDT_SAFE_ADDRESS);
    state.register_fee_market_address(FEE_MARKET_ADDRESS);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.distribute_fee(MultiValueEncoded::new(), Some(CALLER_IS_NOT_OWNER));
}
