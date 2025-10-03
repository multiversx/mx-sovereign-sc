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

    let operation_nonce = state.common_setup.next_operation_nonce();

    state.update_sovereign_config(
        ManagedBuffer::new(),
        new_config,
        operation_nonce,
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

    let operation_nonce = state.common_setup.next_operation_nonce();

    state.update_sovereign_config(
        hash_of_hashes,
        new_config,
        operation_nonce,
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

    let operation_nonce = state.common_setup.next_operation_nonce();

    state.update_sovereign_config(
        hash_of_hashes,
        new_config,
        operation_nonce,
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

    let num_of_validators: u64 = 3;
    let dummy_message = ManagedBuffer::new_from_bytes(&[0x01]);
    let (signature, pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(num_of_validators as usize, &dummy_message);

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

    let bitmap = ManagedBuffer::new_from_bytes(&[0x07]);
    let epoch = 0;

    for id in 2..=num_of_validators {
        let validator_data = ValidatorData {
            id: BigUint::from(id as u32),
            address: OWNER_ADDRESS.to_managed_address(),
            bls_key: pub_keys[(id - 1) as usize].clone(),
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

    assert_eq!(state.common_setup.get_bls_key_id(&new_validator_bls_key), 1);

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
    additional_stake_vec.push(StakeArgs {
        token_identifier: FIRST_TEST_TOKEN.to_token_identifier(),
        amount: stake_amount.clone(),
    });

    let config = SovereignConfig {
        min_validators: 0,
        max_validators: 2,
        min_stake: stake_amount.clone(),
        opt_additional_stake_required: Some(additional_stake_vec),
    };
    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(config), None);

    let payments = combined_stake_payments(&stake_amount);

    let new_validator_bls_key = BLSKey::random();
    state
        .common_setup
        .register(&new_validator_bls_key, &payments, None);
    assert_eq!(state.common_setup.get_bls_key_id(&new_validator_bls_key), 1);

    let owner_initial_egld = BigUint::from(OWNER_BALANCE);
    let owner_initial_token = BigUint::from(ONE_HUNDRED_MILLION);
    let expected_owner_egld = owner_initial_egld.clone() - stake_amount.clone();
    let expected_owner_token = owner_initial_token.clone() - stake_amount.clone();
    assert_contract_and_owner_balances(
        &mut state,
        &stake_amount,
        &stake_amount,
        &expected_owner_egld,
        &expected_owner_token,
    );

    state.common_setup.unregister(&new_validator_bls_key, None);

    let zero = BigUint::zero();
    assert_contract_and_owner_balances(
        &mut state,
        &zero,
        &zero,
        &owner_initial_egld,
        &owner_initial_token,
    );

    assert_eq!(state.common_setup.get_bls_key_id(&new_validator_bls_key), 0);
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

    let payments = single_token_payment(&token_amount);
    let registered_bls_keys = register_validators(&mut state, num_of_validators, &payments);

    let expected_token_amount = token_amount.clone() * num_of_validators;
    let owner_token_after_stake =
        BigUint::from(ONE_HUNDRED_MILLION) - expected_token_amount.clone();
    assert_contract_and_owner_token_balances(
        &mut state,
        &expected_token_amount,
        &owner_token_after_stake,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = full_bitmap(num_of_validators);
    let epoch = 0;

    for (index, validator_bls_key) in registered_bls_keys.iter().enumerate() {
        let validator_id = (index + 1) as u32;

        unregister_validator_via_bridge_operation(
            &mut state,
            validator_id,
            validator_bls_key,
            num_of_validators,
            &bitmap,
            epoch,
        );

        assert_eq!(state.common_setup.get_bls_key_id(validator_bls_key), 0);
    }

    let zero = BigUint::zero();
    let owner_initial_token = BigUint::from(ONE_HUNDRED_MILLION);
    assert_contract_and_owner_token_balances(&mut state, &zero, &owner_initial_token);
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

fn combined_stake_payments(amount: &BigUint<StaticApi>) -> MultiEgldOrEsdtPayment<StaticApi> {
    let mut payments = MultiEgldOrEsdtPayment::new();
    payments.push(EgldOrEsdtTokenPayment::new(
        EgldOrEsdtTokenIdentifier::from(EGLD_000000_TOKEN_IDENTIFIER.as_bytes()),
        0,
        amount.clone(),
    ));
    payments.push(EgldOrEsdtTokenPayment::new(
        EgldOrEsdtTokenIdentifier::from(FIRST_TEST_TOKEN.as_bytes()),
        0,
        amount.clone(),
    ));

    payments
}

fn single_token_payment(amount: &BigUint<StaticApi>) -> MultiEgldOrEsdtPayment<StaticApi> {
    let mut payments = MultiEgldOrEsdtPayment::new();
    payments.push(EgldOrEsdtTokenPayment::new(
        EgldOrEsdtTokenIdentifier::from(FIRST_TEST_TOKEN.as_bytes()),
        0,
        amount.clone(),
    ));

    payments
}

fn assert_contract_and_owner_balances(
    state: &mut ChainConfigTestState,
    contract_egld: &BigUint<StaticApi>,
    contract_token: &BigUint<StaticApi>,
    owner_egld: &BigUint<StaticApi>,
    owner_token: &BigUint<StaticApi>,
) {
    state
        .common_setup
        .world
        .check_account(CHAIN_CONFIG_ADDRESS)
        .balance(contract_egld);
    state
        .common_setup
        .world
        .check_account(CHAIN_CONFIG_ADDRESS)
        .esdt_balance(FIRST_TEST_TOKEN, contract_token);
    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .balance(owner_egld);
    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(FIRST_TEST_TOKEN, owner_token);
}

fn assert_contract_and_owner_token_balances(
    state: &mut ChainConfigTestState,
    contract_token: &BigUint<StaticApi>,
    owner_token: &BigUint<StaticApi>,
) {
    state
        .common_setup
        .world
        .check_account(CHAIN_CONFIG_ADDRESS)
        .esdt_balance(FIRST_TEST_TOKEN, contract_token);
    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(FIRST_TEST_TOKEN, owner_token);
}

fn register_validators(
    state: &mut ChainConfigTestState,
    count: u64,
    payments: &MultiEgldOrEsdtPayment<StaticApi>,
) -> Vec<ManagedBuffer<StaticApi>> {
    let mut bls_keys = Vec::new();

    for expected_id in 1..=count {
        let bls_key = BLSKey::random();
        state.common_setup.register(&bls_key, payments, None);
        assert_eq!(state.common_setup.get_bls_key_id(&bls_key), expected_id);
        bls_keys.push(bls_key);
    }

    bls_keys
}

fn full_bitmap(num_of_validators: u64) -> ManagedBuffer<StaticApi> {
    let mut bitmap_bytes = vec![0u8; num_of_validators.div_ceil(8) as usize];
    for index in 0..num_of_validators {
        let byte_index = (index / 8) as usize;
        let bit_index = (index % 8) as u8;
        bitmap_bytes[byte_index] |= 1u8 << bit_index;
    }

    ManagedBuffer::new_from_bytes(&bitmap_bytes)
}

fn unregister_validator_via_bridge_operation(
    state: &mut ChainConfigTestState,
    validator_id: u32,
    validator_bls_key: &ManagedBuffer<StaticApi>,
    num_of_validators: u64,
    bitmap: &ManagedBuffer<StaticApi>,
    epoch: u64,
) {
    let validator_data = ValidatorData {
        id: BigUint::from(validator_id),
        address: OWNER_ADDRESS.to_managed_address(),
        bls_key: validator_bls_key.clone(),
    };

    let validator_data_hash = validator_data.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&validator_data_hash.to_vec()));
    let (signature, pub_keys) = state
        .common_setup
        .get_sig_and_pub_keys(num_of_validators as usize, &hash_of_hashes);

    state.common_setup.set_bls_keys_in_header_storage(pub_keys);
    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap.clone(),
        epoch,
        MultiValueEncoded::from_iter(vec![validator_data_hash]),
    );

    state.common_setup.unregister_validator(
        &hash_of_hashes,
        validator_data,
        0,
        None,
        Some(EXECUTED_BRIDGE_OP_EVENT),
    );
}
