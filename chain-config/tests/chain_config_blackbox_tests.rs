use chain_config::validator_rules::ValidatorRulesModule;
use chain_config_blackbox_setup::ChainConfigTestState;
use common_test_setup::{constants::CHAIN_CONFIG_ADDRESS, CallerAddress};
use error_messages::{INVALID_MIN_MAX_VALIDATOR_NUMBERS, SETUP_PHASE_NOT_COMPLETED};
use multiversx_sc::{
    imports::OptionalValue,
    types::{BigUint, ManagedBuffer, MultiValueEncoded},
};
use multiversx_sc_scenario::{multiversx_chain_vm::crypto_functions::sha256, ScenarioTxWhitebox};
use structs::{
    configs::SovereignConfig,
    forge::{ContractInfo, ScArray},
    generate_hash::GenerateHash,
};

mod chain_config_blackbox_setup;

// TODO: Add change owner functionality after the fix is done in lock_operation_hash endpoint

/// ### TEST
/// C-CONFIG_DEPLOY_OK
///
/// ### ACTION
/// Deploy chain-config with default config
///
/// ### EXPECTED
/// Chain config is deployed
#[test]
fn test_deploy_chain_config() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);
}

/// ### TEST
/// C-CONFIG_DEPLOY_FAIL
///
/// ### ACTION
/// Call 'update_chain_config_during_setup_phase()' with a new valid config
///
/// ### EXPECTED
/// Chain config is updated with the new config
#[test]
fn test_deploy_chain_config_invalid_config() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig {
        min_validators: 2,
        max_validators: 1,
        ..SovereignConfig::default_config()
    };

    state.common_setup.deploy_chain_config(
        OptionalValue::Some(config),
        Some(INVALID_MIN_MAX_VALIDATOR_NUMBERS),
    );
}

/// ### TEST
/// C-CONFIG_UPDATE_CONFIG_DURING_SETUP_PHASE_OK_003
///
/// ### ACTION
/// Call 'update_chain_config_during_setup_phase()' with a new valid config
///
/// ### EXPECTED
/// Chain config is updated with the new config
#[test]
fn test_update_config_during_setup_phase() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let new_config = SovereignConfig::new(2, 4, BigUint::default(), None);

    state.update_sovereign_config_during_setup_phase(new_config, None);
}

/// ### TEST
/// C-CONFIG_UPDATE_CONFIG_DURING_SETUP_PHASE_FAIL_004
///
/// ### ACTION
/// Call 'update_chain_config_during_setup_phase()' with an new invalid config
///
/// ### EXPECTED
/// Error INVALID_MIN_MAX_VALIDATOR_NUMBERS
#[test]
fn test_update_config_during_setup_phase_wrong_validators_array() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let new_config = SovereignConfig::new(2, 1, BigUint::default(), None);

    state.update_sovereign_config_during_setup_phase(
        new_config,
        Some(INVALID_MIN_MAX_VALIDATOR_NUMBERS),
    );
}

/// ### TEST
/// C-CONFIG_COMPLETE_SETUP_PHASE_OK
///
/// ### ACTION
/// Call 'complete_chain_config_setup_phase()'
///
/// ### EXPECTED
/// Chain config's setup phase is completed
#[test]
fn test_complete_setup_phase() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state.common_setup.complete_chain_config_setup_phase(None);
}

/// ### TEST
/// C-CONFIG_UPDATE_CONFIG_FAIL_006
///
/// ### ACTION
/// Call 'update_sovereign_config()' during the setup phase
///
/// ### EXPECTED
/// Error SETUP_PHASE_NOT_COMPLETED
#[test]
fn test_update_config_setup_phase_not_completed() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .deploy_header_verifier(vec![ContractInfo::new(
            ScArray::ChainConfig,
            CHAIN_CONFIG_ADDRESS.to_managed_address(),
        )]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let new_config = SovereignConfig::new(2, 1, BigUint::default(), None);

    state.update_sovereign_config(
        ManagedBuffer::new(),
        new_config,
        Some(SETUP_PHASE_NOT_COMPLETED),
        None,
    );
}

/// ### TEST
/// C-CONFIG_UPDATE_CONFIG_OK_007
///
/// ### ACTION
/// Call 'update_sovereign_config()'  with an invalid config
///
/// ### EXPECTED
/// failedBridgeOp event is emitted
#[test]
fn test_update_config_invalid_config() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .deploy_header_verifier(vec![ContractInfo::new(
            ScArray::ChainConfig,
            CHAIN_CONFIG_ADDRESS.to_managed_address(),
        )]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let new_config = SovereignConfig::new(2, 1, BigUint::default(), None);

    let config_hash = new_config.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&config_hash.to_vec()));

    state.common_setup.register_operation(
        CallerAddress::Owner,
        ManagedBuffer::new(),
        &hash_of_hashes,
        MultiValueEncoded::from_iter(vec![config_hash]),
    );

    state.common_setup.complete_chain_config_setup_phase(None);

    state.update_sovereign_config(hash_of_hashes, new_config, None, Some("failedBridgeOp"));
}

/// ### TEST
/// C-CONFIG_UPDATE_CONFIG_OK_008
///
/// ### ACTION
/// Call 'update_sovereign_config()'  
///
/// ### EXPECTED
/// executedBridgeOp event is emitted
#[test]
fn test_update_config() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .deploy_header_verifier(vec![ContractInfo::new(
            ScArray::ChainConfig,
            CHAIN_CONFIG_ADDRESS.to_managed_address(),
        )]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let new_config = SovereignConfig::new(1, 2, BigUint::default(), None);

    let config_hash = new_config.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&config_hash.to_vec()));

    state.common_setup.register_operation(
        CallerAddress::Owner,
        ManagedBuffer::new(),
        &hash_of_hashes,
        MultiValueEncoded::from_iter(vec![config_hash]),
    );

    state.common_setup.complete_chain_config_setup_phase(None);

    state.update_sovereign_config(hash_of_hashes, new_config, None, Some("executedBridgeOp"));

    state
        .common_setup
        .world
        .query()
        .to(CHAIN_CONFIG_ADDRESS)
        .whitebox(chain_config::contract_obj, |sc| {
            let config = sc.sovereign_config().get();
            assert!(config.min_validators == 1 && config.max_validators == 2);
        });
}
