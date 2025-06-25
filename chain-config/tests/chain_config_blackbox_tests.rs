use chain_config::validator_rules::ValidatorRulesModule;
use chain_config_blackbox_setup::ChainConfigTestState;
use common_test_setup::constants::{
    CHAIN_CONFIG_ADDRESS, FIRST_TEST_TOKEN, OWNER_ADDRESS, USER_ADDRESS,
};
use error_messages::{
    INVALID_ADDITIONAL_STAKE, INVALID_MIN_MAX_VALIDATOR_NUMBERS, INVALID_REGISTRATION_STATUS,
    SETUP_PHASE_NOT_COMPLETED, VALIDATOR_ALREADY_REGISTERED, VALIDATOR_NOT_REGISTERED,
    VALIDATOR_RANGE_EXCEEDED,
};
use multiversx_sc::{
    imports::OptionalValue,
    types::{
        BigUint, EsdtTokenData, EsdtTokenPayment, ManagedBuffer, ManagedVec, MultiEsdtPayment,
        MultiValueEncoded,
    },
};
use multiversx_sc_scenario::{multiversx_chain_vm::crypto_functions::sha256, ScenarioTxWhitebox};
use setup_phase::SetupPhaseModule;
use structs::{
    configs::{SovereignConfig, StakeArgs},
    forge::ScArray,
    generate_hash::GenerateHash,
    ValidatorInfo,
};

mod chain_config_blackbox_setup;

/// ### TEST
/// C-CONFIG_DEPLOY_OK
///
/// ### ACTION
/// Deploy chain-config with default config
///
/// ### EXPECTED
/// Chain config is deployed
#[test]
fn test_deploy_chain_config_default_config() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);
}

/// ### TEST
/// C-CONFIG_DEPLOY_OK
///
/// ### ACTION
/// Deploy chain-config with specific config
///
/// ### EXPECTED
/// Chain config is deployed
#[test]
fn test_deploy_chain_config() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig::new(1, 2, BigUint::from(100u32), None);

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(config), None);
}

/// ### TEST
/// C-CONFIG_DEPLOY_FAIL
///
/// ### ACTION
/// Call 'update_chain_config_during_setup_phase()' with a invalid config
///
/// ### EXPECTED
/// ERROR INVALID_MIN_MAX_VALIDATOR_NUMBERS
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
/// C-CONFIG_COMPLETE_SETUP_PHASE_OK
///
/// ### ACTION
/// Call `complete_setup_phase()`
///
/// ### EXPECTED
/// Setup phase is completed
#[test]
fn complete_setup_phase() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state.common_setup.complete_chain_config_setup_phase(None);

    state
        .common_setup
        .world
        .query()
        .to(CHAIN_CONFIG_ADDRESS)
        .whitebox(chain_config::contract_obj, |sc| {
            assert!(sc.is_setup_phase_complete());
        })
}

/// ### TEST
/// C-CONFIG_UPDATE_CONFIG_DURING_SETUP_PHASE_OK
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
/// C-CONFIG_UPDATE_CONFIG_DURING_SETUP_PHASE_FAIL
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
/// C-CONFIG_UPDATE_CONFIG_FAIL
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
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

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
/// C-CONFIG_UPDATE_CONFIG_OK
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
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let new_config = SovereignConfig::new(2, 1, BigUint::default(), None);

    let config_hash = new_config.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&config_hash.to_vec()));

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        ManagedBuffer::new(),
        &hash_of_hashes,
        MultiValueEncoded::from_iter(vec![config_hash]),
    );

    state.common_setup.complete_chain_config_setup_phase(None);

    state.update_sovereign_config(hash_of_hashes, new_config, None, Some("failedBridgeOp"));
}

/// ### TEST
/// C-CONFIG_UPDATE_CONFIG_OK
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
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let new_config = SovereignConfig::new(1, 2, BigUint::default(), None);

    let config_hash = new_config.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&config_hash.to_vec()));

    state.common_setup.register_operation(
        OWNER_ADDRESS,
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

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_FAIL
///
/// ### ACTION
/// Call 'register()' with too many validators
///
/// ### EXPECTED
/// Error VALIDATOR_RANGE_EXCEEDED
#[test]
fn test_register_validator_range_exceeded_too_many_validators() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(SovereignConfig::default_config()), None);

    let new_validator_one = ValidatorInfo {
        address: USER_ADDRESS.to_managed_address(),
        bls_key: ManagedBuffer::from("validator1"),
        egld_stake: BigUint::default(),
        token_stake: ManagedVec::new(),
    };

    let new_validator_two = ValidatorInfo {
        address: OWNER_ADDRESS.to_managed_address(),
        bls_key: ManagedBuffer::from("validator2"),
        egld_stake: BigUint::default(),
        token_stake: ManagedVec::new(),
    };

    state.register(
        &new_validator_one,
        MultiEsdtPayment::new(),
        None,
        Some("register"),
    );
    let id_one = state.get_bls_key_id(&new_validator_one.bls_key);
    assert!(state.get_bls_key_by_id(&id_one) == new_validator_one.bls_key);

    state.register(
        &new_validator_two,
        MultiEsdtPayment::new(),
        Some(VALIDATOR_RANGE_EXCEEDED),
        None,
    );
}

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_FAIL
///
/// ### ACTION
/// Call 'register()' with already registered validator
///
/// ### EXPECTED
/// Error VALIDATOR_ALREADY_REGISTERED
#[test]
fn test_register_validator_already_registered() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let new_validator = ValidatorInfo {
        address: USER_ADDRESS.to_managed_address(),
        bls_key: ManagedBuffer::from("validator1"),
        egld_stake: BigUint::default(),
        token_stake: ManagedVec::new(),
    };

    state.register(
        &new_validator,
        MultiEsdtPayment::new(),
        None,
        Some("register"),
    );
    assert!(state.get_bls_key_id(&new_validator.bls_key) == 1);

    state.register(
        &new_validator,
        MultiEsdtPayment::new(),
        Some(VALIDATOR_ALREADY_REGISTERED),
        None,
    );
}

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_FAIL
///
/// ### ACTION
/// Call 'register()' as a validator that isn't whitelisted
///
/// ### EXPECTED
/// Error INVALID_ADDITIONAL_STAKE
#[test]
fn test_register_validator_not_whitelisted() {
    let mut state = ChainConfigTestState::new();

    let first_token_stake_arg = StakeArgs {
        token_id: FIRST_TEST_TOKEN.to_token_identifier(),
        amount: BigUint::from(100u64),
    };

    let additional_stage_args = ManagedVec::from(vec![first_token_stake_arg]);

    let config = SovereignConfig {
        max_validators: 2,
        opt_additional_stake_required: Some(additional_stage_args),
        ..SovereignConfig::default_config()
    };

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(config), None);

    let new_validator = ValidatorInfo {
        address: USER_ADDRESS.to_managed_address(),
        bls_key: ManagedBuffer::from("validator1"),
        egld_stake: BigUint::default(),
        token_stake: ManagedVec::new(),
    };

    state.register(
        &new_validator,
        MultiEsdtPayment::new(),
        Some(INVALID_ADDITIONAL_STAKE),
        None,
    );
}

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_OK
///
/// ### ACTION
/// Call 'register()' as a whitelisted validator
///
/// ### EXPECTED
/// Validator is registered successfully
#[test]
fn test_register_validator_is_whitelisted() {
    let mut state = ChainConfigTestState::new();

    let first_token_stake_arg = StakeArgs {
        token_id: FIRST_TEST_TOKEN.to_token_identifier(),
        amount: BigUint::from(100u64),
    };

    let additional_stage_args = ManagedVec::from(vec![first_token_stake_arg]);

    let config = SovereignConfig {
        max_validators: 2,
        opt_additional_stake_required: Some(additional_stage_args),
        ..SovereignConfig::default_config()
    };

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(config), None);

    let new_validator = ValidatorInfo {
        address: USER_ADDRESS.to_managed_address(),
        bls_key: ManagedBuffer::from("validator1"),
        egld_stake: BigUint::default(),
        token_stake: ManagedVec::new(),
    };

    let payment = EsdtTokenPayment::new(
        FIRST_TEST_TOKEN.to_token_identifier(),
        0,
        BigUint::from(100u64),
    );

    state.register(
        &new_validator,
        ManagedVec::from(vec![payment]),
        None,
        Some("register"),
    );
}

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_OK
///
/// ### ACTION
/// Call 'register()' as a non whitelisted validator after genesis phase
///
/// ### EXPECTED
/// Validator is registered successfully
#[test]
fn test_register_validator_not_whitelisted_after_genesis() {
    let mut state = ChainConfigTestState::new();

    let first_token_stake_arg = StakeArgs {
        token_id: FIRST_TEST_TOKEN.to_token_identifier(),
        amount: BigUint::from(100u64),
    };

    let additional_stage_args = ManagedVec::from(vec![first_token_stake_arg]);

    let config = SovereignConfig {
        max_validators: 2,
        opt_additional_stake_required: Some(additional_stage_args),
        ..SovereignConfig::default_config()
    };

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(config), None);

    let whitelisted_validator = ValidatorInfo {
        address: USER_ADDRESS.to_managed_address(),
        bls_key: ManagedBuffer::from("validator1"),
        egld_stake: BigUint::default(),
        token_stake: ManagedVec::new(),
    };

    let payment = EsdtTokenPayment::new(
        FIRST_TEST_TOKEN.to_token_identifier(),
        0,
        BigUint::from(100u64),
    );

    state.register(
        &whitelisted_validator,
        ManagedVec::from(vec![payment]),
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

    let new_status_hash_byte_array = sha256(&[1u8]);
    let new_status_hash = ManagedBuffer::new_from_bytes(&new_status_hash_byte_array);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&new_status_hash_byte_array));

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        ManagedBuffer::new(),
        &hash_of_hashes,
        MultiValueEncoded::from_iter(vec![new_status_hash]),
    );

    state.common_setup.update_registration_status(
        &hash_of_hashes,
        1,
        None,
        Some("registrationStatusUpdate"),
    );

    let validator = ValidatorInfo {
        address: USER_ADDRESS.to_managed_address(),
        bls_key: ManagedBuffer::from("validator2"),
        egld_stake: BigUint::default(),
        token_stake: ManagedVec::new(),
    };

    state.register(&validator, ManagedVec::new(), None, Some("register"));
}

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_ERROR
///
/// ### ACTION
/// Call 'register()' as a non whitelisted validator during genesis phase
///
/// ### EXPECTED
/// Error GENESIS_PHASE_NOT_COMPLETE
#[test]
fn test_register_validator_not_whitelisted_during_genesis() {
    let mut state = ChainConfigTestState::new();

    let first_token_stake_arg = StakeArgs {
        token_id: FIRST_TEST_TOKEN.to_token_identifier(),
        amount: BigUint::from(100u64),
    };

    let additional_stage_args = ManagedVec::from(vec![first_token_stake_arg]);

    let config = SovereignConfig {
        max_validators: 2,
        opt_additional_stake_required: Some(additional_stage_args),
        ..SovereignConfig::default_config()
    };

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(config), None);

    let whitelisted_validator = ValidatorInfo {
        address: USER_ADDRESS.to_managed_address(),
        bls_key: ManagedBuffer::from("validator1"),
        egld_stake: BigUint::default(),
        token_stake: ManagedVec::new(),
    };

    let payment = EsdtTokenPayment::new(
        FIRST_TEST_TOKEN.to_token_identifier(),
        0,
        BigUint::from(100u64),
    );

    state.register(
        &whitelisted_validator,
        ManagedVec::from(vec![payment]),
        None,
        Some("register"),
    );

    let validator = ValidatorInfo {
        address: USER_ADDRESS.to_managed_address(),
        bls_key: ManagedBuffer::from("validator2"),
        egld_stake: BigUint::default(),
        token_stake: ManagedVec::new(),
    };

    state.register(
        &validator,
        ManagedVec::new(),
        Some(INVALID_ADDITIONAL_STAKE),
        None,
    );
}

/// ### TEST
/// C-CONFIG_UNREGISTER_FAIL
///
/// ### ACTION
/// Call 'unregister()' with not registered validator
///
/// ### EXPECTED
/// Error VALIDATOR_NOT_REGISTERED
#[test]
fn test_unregister_validator_not_registered() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig {
        min_validators: 0,
        max_validators: 2,
        ..SovereignConfig::default_config()
    };

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(config), None);

    state.common_setup.complete_chain_config_setup_phase(None);

    let new_validator = ValidatorInfo {
        address: USER_ADDRESS.to_managed_address(),
        bls_key: ManagedBuffer::from("validator1"),
        egld_stake: BigUint::default(),
        token_stake: ManagedVec::new(),
    };

    state.unregister(&new_validator, Some(VALIDATOR_NOT_REGISTERED), None);

    assert!(state.get_bls_key_id(&new_validator.bls_key) == 0);
}

/// ### TEST
/// C-CONFIG_UNREGISTER_OK
///
/// ### ACTION
/// Call 'unregister()' with registered validator
///
/// ### EXPECTED
/// Validator is unregistered successfully
#[test]
fn test_unregister_validator() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let new_validator = ValidatorInfo {
        address: USER_ADDRESS.to_managed_address(),
        bls_key: ManagedBuffer::from("validator1"),
        egld_stake: BigUint::default(),
        token_stake: ManagedVec::new(),
    };

    state.register(
        &new_validator,
        MultiEsdtPayment::new(),
        None,
        Some("register"),
    );
    assert!(state.get_bls_key_id(&new_validator.bls_key) == 1);

    state.unregister(&new_validator, None, Some("unregister"));

    assert!(state.get_bls_key_id(&new_validator.bls_key) == 0);
}

/// ### TEST
/// C-CONFIG_UPDATE_REGISTRATION_FAIL
///
/// ### ACTION
/// Call 'update_registration_status()' before setup phase completion
///
/// ### EXPECTED
/// Error SETUP_PHASE_NOT_COMPLETED
#[test]
fn update_registration_setup_not_complete() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state.common_setup.update_registration_status(
        &ManagedBuffer::new(),
        1,
        Some(SETUP_PHASE_NOT_COMPLETED),
        None,
    );
}

/// ### TEST
/// C-CONFIG_UPDATE_REGISTRATION_FAIL
///
/// ### ACTION
/// Call 'update_registration_status()' with the wrong registration status
///
/// ### EXPECTED
/// "failedBridgeOp" event is emitted
#[test]
fn update_registration_wrong_status() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state.common_setup.complete_chain_config_setup_phase(None);

    state.common_setup.update_registration_status(
        &ManagedBuffer::new(),
        1,
        None,
        Some("failedBridgeOp"),
    );
}

/// ### TEST
/// C-CONFIG_UPDATE_REGISTRATION_OK
///
/// ### ACTION
/// Call 'update_registration_status()'
///
/// ### EXPECTED
/// "registrationStatusUpdate" event is emitted and storage is updated
#[test]
fn update_registration_operation_not_registered() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state.common_setup.complete_chain_config_setup_phase(None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let new_status_hash_byte_array = sha256(&[1u8]);
    let new_status_hash = ManagedBuffer::new_from_bytes(&new_status_hash_byte_array);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&new_status_hash_byte_array));

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        ManagedBuffer::new(),
        &hash_of_hashes,
        MultiValueEncoded::from_iter(vec![new_status_hash]),
    );

    state.common_setup.update_registration_status(
        &hash_of_hashes,
        1,
        None,
        Some("registrationStatusUpdate"),
    );

    state
        .common_setup
        .world
        .query()
        .to(CHAIN_CONFIG_ADDRESS)
        .whitebox(chain_config::contract_obj, |sc| {
            assert!(sc.registration_status().get() == 1);
        })
}
