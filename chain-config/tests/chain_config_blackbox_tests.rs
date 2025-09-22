use chain_config::storage::ChainConfigStorageModule;
use chain_config_blackbox_setup::ChainConfigTestState;
use common_test_setup::base_setup::helpers::BLSKey;
use common_test_setup::constants::{
    CHAIN_CONFIG_ADDRESS, EXECUTED_BRIDGE_OP_EVENT, FIRST_TEST_TOKEN, ONE_HUNDRED_MILLION,
    OWNER_ADDRESS, OWNER_BALANCE, USER_ADDRESS,
};
use error_messages::{
    ADDITIONAL_STAKE_ZERO_VALUE, CHAIN_CONFIG_SETUP_PHASE_NOT_COMPLETE, INVALID_ADDITIONAL_STAKE,
    INVALID_BLS_KEY_FOR_CALLER, INVALID_BLS_KEY_PROVIDED, INVALID_EGLD_STAKE,
    INVALID_MIN_MAX_VALIDATOR_NUMBERS, REGISTRATIONS_DISABLED_GENESIS_PHASE,
    SETUP_PHASE_NOT_COMPLETED, VALIDATOR_ALREADY_REGISTERED, VALIDATOR_NOT_REGISTERED,
    VALIDATOR_RANGE_EXCEEDED,
};
use multiversx_sc::{
    chain_core::EGLD_000000_TOKEN_IDENTIFIER,
    imports::OptionalValue,
    types::{
        BigUint, EgldOrEsdtTokenIdentifier, EgldOrEsdtTokenPayment, ManagedBuffer, ManagedVec,
        MultiEgldOrEsdtPayment, MultiValueEncoded,
    },
};
use multiversx_sc_scenario::api::StaticApi;
use multiversx_sc_scenario::{multiversx_chain_vm::crypto_functions::sha256, ScenarioTxWhitebox};
use setup_phase::SetupPhaseModule;
use structs::{
    configs::{SovereignConfig, StakeArgs},
    forge::ScArray,
    generate_hash::GenerateHash,
    ValidatorData,
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
fn test_complete_setup_phase() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .register(&BLSKey::random(), &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

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
/// Call 'update_config()' with additional stake with a zero amount
///
/// ### EXPECTED
/// Error ADDITIONAL_STAKE_ZERO_VALUE
#[test]
fn test_update_config_during_setup_phase_additional_stake_zero_amount() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let first_token_stake_arg = StakeArgs {
        token_identifier: FIRST_TEST_TOKEN.to_token_identifier(),
        amount: BigUint::zero(),
    };

    let additional_stage_args = ManagedVec::from(vec![first_token_stake_arg]);

    let new_config = SovereignConfig::new(2, 4, BigUint::default(), Some(additional_stage_args));

    state.update_sovereign_config_during_setup_phase(new_config, Some(ADDITIONAL_STAKE_ZERO_VALUE));
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
        .complete_header_verifier_setup_phase(Some(CHAIN_CONFIG_SETUP_PHASE_NOT_COMPLETE));

    let new_config = SovereignConfig::new(2, 1, BigUint::default(), None);

    state.update_sovereign_config(
        ManagedBuffer::new(),
        new_config,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(SETUP_PHASE_NOT_COMPLETED),
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

    let new_config = SovereignConfig::new(2, 1, BigUint::default(), None);
    let config_hash = new_config.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&config_hash.to_vec()));
    let (signature, pub_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);
    let bitmap = ManagedBuffer::new_from_bytes(&[0x01]);

    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        0,
        MultiValueEncoded::from_iter(vec![config_hash]),
    );

    state.update_sovereign_config(
        hash_of_hashes,
        new_config,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(INVALID_MIN_MAX_VALIDATOR_NUMBERS),
    );
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

    let new_config = SovereignConfig::new(1, 2, BigUint::default(), None);
    let config_hash = new_config.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&config_hash.to_vec()));
    let (signature, pub_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);
    let bitmap = ManagedBuffer::new_from_bytes(&[0x01]);

    state
        .common_setup
        .register(&pub_keys[0], &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        0,
        MultiValueEncoded::from_iter(vec![config_hash]),
    );

    state.common_setup.complete_chain_config_setup_phase();

    state.update_sovereign_config(
        hash_of_hashes,
        new_config,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        None,
    );

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
fn test_register_range_exceeded_too_many_validators() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(SovereignConfig::default_config()), None);

    let payments_vec = MultiEgldOrEsdtPayment::new();

    let new_validator_one = BLSKey::random();
    let new_validator_two = BLSKey::random();
    let new_validator_three = BLSKey::random();

    state
        .common_setup
        .register(&new_validator_one, &payments_vec, None);
    let id_one = state.common_setup.get_bls_key_id(&new_validator_one);
    assert!(state.get_bls_key_by_id(&id_one) == new_validator_one);

    state
        .common_setup
        .register(&new_validator_two, &payments_vec, None);
    let id_two = state.common_setup.get_bls_key_id(&new_validator_two);
    assert!(state.get_bls_key_by_id(&id_two) == new_validator_two);

    state.common_setup.register(
        &new_validator_three,
        &payments_vec,
        Some(VALIDATOR_RANGE_EXCEEDED),
    );
}

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_FAIL
///
/// ### ACTION
/// Call 'register()' with not enough EGLD stake
///
/// ### EXPECTED
/// Error INVALID_EGLD_STAKE
#[test]
fn test_register_not_enough_egld_stake() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig {
        max_validators: 3,
        min_stake: BigUint::from(100u64),
        ..SovereignConfig::default_config()
    };

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(config), None);

    let egld_payment_not_enough = EgldOrEsdtTokenPayment::new(
        EgldOrEsdtTokenIdentifier::from(EGLD_000000_TOKEN_IDENTIFIER.as_bytes()),
        0,
        BigUint::from(99u64),
    );

    let mut payments_vec_not_enough = MultiEgldOrEsdtPayment::new();
    payments_vec_not_enough.push(egld_payment_not_enough);

    state.common_setup.register(
        &BLSKey::random(),
        &payments_vec_not_enough,
        Some(INVALID_EGLD_STAKE),
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
fn test_register_already_registered() {
    let mut state = ChainConfigTestState::new();

    let sovereign_config = SovereignConfig {
        max_validators: 10,
        ..SovereignConfig::default_config()
    };
    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(sovereign_config), None);

    let payments_vec = MultiEgldOrEsdtPayment::new();

    state
        .common_setup
        .register(&BLSKey::random(), &payments_vec, None);

    let new_validator = BLSKey::random();
    state
        .common_setup
        .register(&new_validator, &payments_vec, None);
    assert!(state.common_setup.get_bls_key_id(&new_validator) == 2);

    state.common_setup.register(
        &new_validator,
        &payments_vec,
        Some(VALIDATOR_ALREADY_REGISTERED),
    );
}

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_FAIL
///
/// ### ACTION
/// Call 'register()' with invalid BLS key
///
/// ### EXPECTED
/// Error INVALID_BLS_KEY_PROVIDED
#[test]
fn test_register_invalid_bls_key() {
    let mut state = ChainConfigTestState::new();
    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state.common_setup.register(
        &ManagedBuffer::from("invalid bls key"),
        &MultiEgldOrEsdtPayment::new(),
        Some(INVALID_BLS_KEY_PROVIDED),
    );
}

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_FAIL
///
/// ### ACTION
/// Call 'register()' after genesis phase
///
/// ### EXPECTED
/// Error REGISTRATIONS_DISABLED_GENESIS_PHASE
#[test]
fn test_register_after_genesis() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .register(&BLSKey::random(), &ManagedVec::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.register(
        &BLSKey::random(),
        &ManagedVec::new(),
        Some(REGISTRATIONS_DISABLED_GENESIS_PHASE),
    );
}

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_OK
///
/// ### ACTION
/// Call 'register_validator()' after genesis phase
///
/// ### EXPECTED
/// Validator is registered successfully after genesis
#[test]
fn test_register_validator_after_genesis() {
    let mut state = ChainConfigTestState::new();

    let first_token_stake_arg = StakeArgs {
        token_identifier: FIRST_TEST_TOKEN.to_token_identifier(),
        amount: BigUint::from(100u64),
    };

    let additional_stage_args = ManagedVec::from(vec![first_token_stake_arg]);

    let config = SovereignConfig {
        max_validators: 3,
        opt_additional_stake_required: Some(additional_stage_args),
        ..SovereignConfig::default_config()
    };

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(config), None);

    let payment = EgldOrEsdtTokenPayment::new(
        EgldOrEsdtTokenIdentifier::from(FIRST_TEST_TOKEN.as_bytes()),
        0,
        BigUint::from(100u64),
    );

    let mut payments_vec = MultiEgldOrEsdtPayment::new();
    payments_vec.push(payment);

    let (signature, pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(1, &ManagedBuffer::new());

    state
        .common_setup
        .register(&pub_keys[0], &payments_vec, None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = ManagedBuffer::new_from_bytes(&[0x06]);
    let epoch = 0;

    for id in 2..4 {
        let validator_data = ValidatorData {
            id: BigUint::from(id as u32),
            address: OWNER_ADDRESS.to_managed_address(),
            bls_key: pub_keys[1].clone(),
        };
        state.common_setup.register_validator_operation(
            validator_data,
            signature.clone(),
            bitmap.clone(),
            epoch,
        );
    }
}

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_ERROR
///
/// ### ACTION
/// Call 'register()' twice with and without additional stake
///
/// ### EXPECTED
/// Successful register and INVALID_ADDITIONAL_STAKE
#[test]
fn test_register_additional_stake() {
    let mut state = ChainConfigTestState::new();

    let first_token_stake_arg = StakeArgs {
        token_identifier: FIRST_TEST_TOKEN.to_token_identifier(),
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

    let payment = EgldOrEsdtTokenPayment::new(
        EgldOrEsdtTokenIdentifier::from(FIRST_TEST_TOKEN.as_bytes()),
        0,
        BigUint::from(100u64),
    );

    let mut payments_with_additional_stake = MultiEgldOrEsdtPayment::new();
    payments_with_additional_stake.push(payment);
    state
        .common_setup
        .register(&BLSKey::random(), &payments_with_additional_stake, None);

    let payments_no_additional_stake = MultiEgldOrEsdtPayment::new();
    state.common_setup.register(
        &BLSKey::random(),
        &payments_no_additional_stake,
        Some(INVALID_ADDITIONAL_STAKE),
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
fn test_unregister_not_registered() {
    let mut state = ChainConfigTestState::new();
    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let bls_key = BLSKey::random();
    state
        .common_setup
        .unregister(&bls_key, Some(VALIDATOR_NOT_REGISTERED));

    assert!(state.common_setup.get_bls_key_id(&bls_key) == 0);
}

/// ### TEST
/// C-CONFIG_UNREGISTER_FAIL
///
/// ### ACTION
/// Call 'unregister()' with registered BLS key but wrong caller
///
/// ### EXPECTED
/// Error INVALID_BLS_KEY_FOR_CALLER
#[test]
fn test_unregister_wrong_caller_for_bls_key() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let new_validator_bls_key = BLSKey::random();
    state
        .common_setup
        .register(&new_validator_bls_key, &ManagedVec::new(), None);

    assert!(state.common_setup.get_bls_key_id(&new_validator_bls_key) == 1);

    state.unregister_with_caller(
        &new_validator_bls_key,
        USER_ADDRESS,
        Some(INVALID_BLS_KEY_FOR_CALLER),
        None,
    );
}

/// ### TEST
/// C-CONFIG_REGISTER_VALIDATOR_FAIL
///
/// ### ACTION
/// Call 'unregister()' with invalid BLS key
///
/// ### EXPECTED
/// Error INVALID_BLS_KEY_PROVIDED
#[test]
fn test_unregister_invalid_bls_key() {
    let mut state = ChainConfigTestState::new();
    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state.common_setup.unregister(
        &ManagedBuffer::from("invalid bls key"),
        Some(INVALID_BLS_KEY_PROVIDED),
    );
}

/// ### TEST
/// C-CONFIG_UNREGISTER_OK
///
/// ### ACTION
/// Call 'unregister()' with registered validator, no stake required
///
/// ### EXPECTED
/// Validator is unregistered successfully
#[test]
fn test_unregister_no_stake() {
    let mut state = ChainConfigTestState::new();
    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let validator_bls_key = BLSKey::random();
    state
        .common_setup
        .register(&validator_bls_key, &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.unregister(&validator_bls_key, None);
}

/// ### TEST
/// C-CONFIG_UNREGISTER_OK
///
/// ### ACTION
/// Call 'unregister()' with registered validator with both EGLD and ESDT stake
///
/// ### EXPECTED
/// Validator is unregistered successfully and stake is returned
#[test]
fn test_unregister() {
    let mut state = ChainConfigTestState::new();

    let stake_amount = BigUint::from(100_000u64);
    let mut additional_stake_vec = ManagedVec::new();
    let additional_stake = StakeArgs {
        token_identifier: FIRST_TEST_TOKEN.to_token_identifier(),
        amount: stake_amount.clone(),
    };
    additional_stake_vec.push(additional_stake);

    let config = SovereignConfig {
        min_validators: 0,
        max_validators: 2,
        min_stake: stake_amount.clone(),
        opt_additional_stake_required: Some(additional_stake_vec),
    };
    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(config), None);

    let payment = EgldOrEsdtTokenPayment::new(
        EgldOrEsdtTokenIdentifier::from(EGLD_000000_TOKEN_IDENTIFIER.as_bytes()),
        0,
        stake_amount.clone(),
    );
    let first_token_payment = EgldOrEsdtTokenPayment::new(
        EgldOrEsdtTokenIdentifier::from(FIRST_TEST_TOKEN.as_bytes()),
        0,
        stake_amount.clone(),
    );

    let mut payments_vec = MultiEgldOrEsdtPayment::new();
    payments_vec.push(payment);
    payments_vec.push(first_token_payment);

    let new_validator_bls_key = BLSKey::random();
    state
        .common_setup
        .register(&new_validator_bls_key, &payments_vec, None);
    assert!(state.common_setup.get_bls_key_id(&new_validator_bls_key) == 1);

    state
        .common_setup
        .world
        .check_account(CHAIN_CONFIG_ADDRESS)
        .balance(&stake_amount);
    state
        .common_setup
        .world
        .check_account(CHAIN_CONFIG_ADDRESS)
        .esdt_balance(FIRST_TEST_TOKEN, &stake_amount);
    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .balance(&(BigUint::from(OWNER_BALANCE) - stake_amount.clone()));
    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            FIRST_TEST_TOKEN,
            &(BigUint::from(ONE_HUNDRED_MILLION) - stake_amount),
        );

    state.common_setup.unregister(&new_validator_bls_key, None);

    state
        .common_setup
        .world
        .check_account(CHAIN_CONFIG_ADDRESS)
        .balance(BigUint::zero());
    state
        .common_setup
        .world
        .check_account(CHAIN_CONFIG_ADDRESS)
        .esdt_balance(FIRST_TEST_TOKEN, BigUint::zero());
    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .balance(BigUint::from(OWNER_BALANCE));
    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(FIRST_TEST_TOKEN, BigUint::from(ONE_HUNDRED_MILLION));

    assert!(state.common_setup.get_bls_key_id(&new_validator_bls_key) == 0);
}

/// ### TEST
/// C-CONFIG_UNREGISTER_VALIDATOR_FAIL
///
/// ### ACTION
/// Call 'unregister()' after genesis phase
///
/// ### EXPECTED
/// Error REGISTRATIONS_DISABLED_GENESIS_PHASE
#[test]
fn test_unregister_after_genesis() {
    let mut state = ChainConfigTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .register(&BLSKey::random(), &ManagedVec::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.unregister(
        &BLSKey::random(),
        Some(REGISTRATIONS_DISABLED_GENESIS_PHASE),
    );
}

/// ### TEST
/// C-CONFIG_UNREGISTER_VALIDATOR_OK
///
/// ### ACTION
/// Call 'unregister_validator()' after genesis phase
///
/// ### EXPECTED
/// Validator is unregistered successfully after genesis
#[test]
fn test_unregister_validator_after_genesis() {
    // register with 3 random validators
    // create the signature and pub keys
    // modify storage with the created pub keys
    let mut state = ChainConfigTestState::new();

    let token_amount = BigUint::from(100_000u64);
    let first_token_stake_arg = StakeArgs {
        token_identifier: FIRST_TEST_TOKEN.to_token_identifier(),
        amount: token_amount.clone(),
    };

    let num_of_validators = 3;
    let additional_stage_args = ManagedVec::from(vec![first_token_stake_arg]);
    let config = SovereignConfig {
        max_validators: num_of_validators,
        opt_additional_stake_required: Some(additional_stage_args),
        ..SovereignConfig::default_config()
    };

    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(config), None);

    let payment = EgldOrEsdtTokenPayment::new(
        EgldOrEsdtTokenIdentifier::from(FIRST_TEST_TOKEN.as_bytes()),
        0,
        token_amount.clone(),
    );
    let mut payments_vec = MultiEgldOrEsdtPayment::new();
    payments_vec.push(payment);

    let mut registered_bls_keys: ManagedVec<StaticApi, ManagedBuffer<StaticApi>> =
        ManagedVec::new();
    for id in 1..(num_of_validators + 1) {
        let validator_bls_key = BLSKey::random();
        registered_bls_keys.push(validator_bls_key.clone());
        state
            .common_setup
            .register(&validator_bls_key, &payments_vec, None);

        assert_eq!(state.common_setup.get_bls_key_id(&validator_bls_key), id);
    }

    let expected_token_amount = token_amount * num_of_validators;
    state
        .common_setup
        .world
        .check_account(CHAIN_CONFIG_ADDRESS)
        .esdt_balance(FIRST_TEST_TOKEN, &expected_token_amount);
    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            FIRST_TEST_TOKEN,
            &(BigUint::from(ONE_HUNDRED_MILLION) - expected_token_amount),
        );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let signature = ManagedBuffer::new();
    let bitmap = ManagedBuffer::new_from_bytes(&num_of_validators.to_be_bytes());
    let epoch = 0;

    for id in 1..4 {
        let validator_bls_key = registered_bls_keys.get(id - 1);

        let validator_data = ValidatorData {
            id: BigUint::from(id as u32), // IDs start from 1
            address: OWNER_ADDRESS.to_managed_address(),
            bls_key: validator_bls_key.clone(),
        };
        state.common_setup.unregister_validator_operation(
            validator_data,
            signature.clone(),
            bitmap.clone(),
            epoch,
        );
    }

    state
        .common_setup
        .world
        .check_account(CHAIN_CONFIG_ADDRESS)
        .esdt_balance(FIRST_TEST_TOKEN, BigUint::zero());
    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(FIRST_TEST_TOKEN, BigUint::from(ONE_HUNDRED_MILLION));
}

/// ### TEST
/// C-CONFIG_UNREGISTER_VALIDATOR_FAIL
///
/// ### ACTION
/// Call 'unregister_validator()' after genesis with invalid data
///
/// ### EXPECTED
/// Errors: VALIDATOR_ID_NOT_REGISTERED and INVALID_VALIDATOR_DATA
#[test]
fn test_unregister_validator_invalid() {
    let mut state = ChainConfigTestState::new();
    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

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

    let signature = ManagedBuffer::new();
    let bitmap = ManagedBuffer::new_from_bytes(&[0x01]);
    let epoch = 0;

    // invalid validator id
    let validator1_bls_key = BLSKey::random();
    let validator_data_invalid_id = ValidatorData {
        id: BigUint::from(999u32),
        address: OWNER_ADDRESS.to_managed_address(),
        bls_key: validator1_bls_key.clone(),
    };
    state.common_setup.unregister_validator_operation(
        validator_data_invalid_id,
        signature.clone(),
        bitmap.clone(),
        epoch,
    );

    assert_eq!(state.common_setup.get_bls_key_id(&validator1_bls_key), 0);

    // invalid validator address for id
    let validator2_bls_key = BLSKey::random();
    let validator_data_invalid_address = ValidatorData {
        id: BigUint::from(1u32),
        address: USER_ADDRESS.to_managed_address(),
        bls_key: validator2_bls_key.clone(),
    };
    state.common_setup.unregister_validator_operation(
        validator_data_invalid_address,
        signature.clone(),
        bitmap.clone(),
        epoch,
    );

    assert_eq!(state.common_setup.get_bls_key_id(&validator2_bls_key), 0);
}
