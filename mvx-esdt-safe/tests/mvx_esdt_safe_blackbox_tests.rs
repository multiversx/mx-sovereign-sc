use common_test_setup::constants::{
    CHAIN_CONFIG_ADDRESS, CROWD_TOKEN_ID, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_TOKEN,
    FIRST_TEST_TOKEN, HEADER_VERIFIER_ADDRESS, ONE_HUNDRED_MILLION, ONE_HUNDRED_THOUSAND,
    OWNER_ADDRESS, SECOND_TEST_TOKEN, SOV_TOKEN, TESTING_SC_ADDRESS, USER_ADDRESS,
};
use common_test_setup::{CallerAddress, RegisterTokenArgs};
use cross_chain::storage::CrossChainStorage;
use cross_chain::{DEFAULT_ISSUE_COST, MAX_GAS_PER_TRANSACTION};
use error_messages::{
    BANNED_ENDPOINT_NAME, CANNOT_REGISTER_TOKEN, DEPOSIT_OVER_MAX_AMOUNT, ERR_EMPTY_PAYMENTS,
    GAS_LIMIT_TOO_HIGH, INVALID_TYPE, MAX_GAS_LIMIT_PER_TX_EXCEEDED, MINT_AND_BURN_ROLES_NOT_FOUND,
    NOTHING_TO_TRANSFER, NO_ESDT_SAFE_ADDRESS, PAYMENT_DOES_NOT_COVER_FEE,
    SETUP_PHASE_ALREADY_COMPLETED, TOKEN_ID_IS_NOT_TRUSTED, TOKEN_IS_FROM_SOVEREIGN,
    TOO_MANY_TOKENS,
};
use header_verifier::OperationHashStatus;
use multiversx_sc::types::MultiValueEncoded;
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EsdtTokenData, EsdtTokenPayment, EsdtTokenType, ManagedBuffer, ManagedVec,
        TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_scenario::{api::StaticApi, ScenarioTxWhitebox};
use mvx_esdt_safe::bridging_mechanism::{BridgingMechanism, TRUSTED_TOKEN_IDS};
use mvx_esdt_safe_blackbox_setup::MvxEsdtSafeTestState;
use setup_phase::SetupPhaseModule;
use structs::configs::{MaxBridgedAmount, SovereignConfig};
use structs::fee::{FeeStruct, FeeType};
use structs::operation::TransferData;
use structs::{
    aliases::PaymentsVec,
    configs::EsdtSafeConfig,
    operation::{Operation, OperationData, OperationEsdtPayment},
};
mod mvx_esdt_safe_blackbox_setup;

/// ### TEST
/// M-ESDT_DEPLOY_OK_001
///
/// ### ACTION
/// Call 'deploy_mvx_esdt_safe()' with default config
///
/// ### EXPECTED
/// Contract is deployed with the default config
#[test]
fn test_deploy() {
    let mut state = MvxEsdtSafeTestState::new();

    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
}

/// ### TEST
/// M-ESDT_DEPLOY_FAIL_002
///
/// ### ACTION
/// Call 'update_configuration()' with invalid config
///
/// ### EXPECTED
/// Error MAX_GAS_LIMIT_PER_TX_EXCEEDED
#[test]
fn test_deploy_invalid_config() {
    let mut state = MvxEsdtSafeTestState::new();

    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        MAX_GAS_PER_TRANSACTION + 1,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    state.update_configuration(
        &ManagedBuffer::new(),
        config,
        Some(MAX_GAS_LIMIT_PER_TX_EXCEEDED),
    );
}

/// ### TEST
/// M-ESDT_REG_FAIL_003
///
/// ### ACTION
/// Call 'register_token()' with invalid token type
///
/// ### EXPECTED
/// Error CANNOT_REGISTER_TOKEN
#[test]
fn test_register_token_invalid_type() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, config);

    let sov_token_id = FIRST_TEST_TOKEN;
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = FIRST_TEST_TOKEN.as_str();
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(
        register_token_args,
        egld_payment,
        Some(CANNOT_REGISTER_TOKEN),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(SECOND_TEST_TOKEN.as_str());
}

/// ### TEST
/// M-ESDT_REG_FAIL_004
///
/// ### ACTION
/// Call 'register_token()' with invalid token type and prefix
///
/// ### EXPECTED
/// Error INVALID_TYPE
#[test]
fn test_register_token_invalid_type_with_prefix() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = SOV_TOKEN;
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = FIRST_TEST_TOKEN.as_str();
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, Some(INVALID_TYPE));

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(SECOND_TEST_TOKEN.as_str());
}

/// ### TEST
/// M-ESDT_REG_FAIL_005
///
/// ### ACTION
/// Call 'register_token()' with token id not starting with prefix
///
/// ### EXPECTED
/// Error CANNOT_REGISTER_TOKEN
#[test]
fn test_register_token_not_native() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = SECOND_TEST_TOKEN;
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = FIRST_TEST_TOKEN.as_str();
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(
        register_token_args,
        egld_payment,
        Some(CANNOT_REGISTER_TOKEN),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(SECOND_TEST_TOKEN.as_str());
}

/// ### TEST
/// M-ESDT_REG_OK_006
///
/// ### ACTION
/// Call 'register_token()' with valid token id and type
///
/// ### EXPECTED
/// The token is registered
#[test]
fn test_register_token_fungible_token() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = SOV_TOKEN;
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = FIRST_TEST_TOKEN.as_str();
    let num_decimals = 3;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, None);

    // TODO: add check for storage after callback fix
}

/// ### TEST
/// M-ESDT_REG_FAIL_007
///
/// ### ACTION
/// Call 'register_token()' with token id not starting with prefix and token type NonFungible
///
/// ### EXPECTED
/// Error CANNOT_REGISTER_TOKEN
#[test]
fn test_register_token_nonfungible_token() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = FIRST_TEST_TOKEN;
    let token_type = EsdtTokenType::NonFungible;
    let token_display_name = "TokenOne";
    let num_decimals = 0;
    let token_ticker = FIRST_TEST_TOKEN.as_str();
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(
        register_token_args,
        egld_payment,
        Some(CANNOT_REGISTER_TOKEN),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(SECOND_TEST_TOKEN.as_str());
}

/// ### TEST
/// M-ESDT_DEP_FAIL_008
///
/// ### ACTION
/// Call 'deposit()' with empty payments_vec and no transfer_data
///
/// ### EXPECTED
/// Error NOTHING_TO_TRANSFER
#[test]
fn test_deposit_nothing_to_transfer() {
    let mut state = MvxEsdtSafeTestState::new();

    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::new(),
        Some(NOTHING_TO_TRANSFER),
        None,
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(FIRST_TEST_TOKEN.as_str());
}

/// ### TEST
/// M-ESDT_SETUP_OK_009
///
/// ### ACTION
/// Call 'complete_setup_phase()'
///
/// ### EXPECTED
/// The setup phase is marked as completed in the smart contract's storage
#[test]
fn test_complete_setup_phase() {
    let mut state = MvxEsdtSafeTestState::new();

    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(
        FIRST_TEST_TOKEN.as_str(),
        token_display_name,
        egld_payment,
        None,
    );
    state.complete_setup_phase(None, Some("unpauseContract"));

    state
        .common_setup
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc.is_setup_phase_complete());
        });
}

/// ### TEST
/// M-ESDT_SETUP_FAIL_010
///
/// ### ACTION
/// Call 'complete_setup_phase()' twice
///
/// ### EXPECTED
/// Error SETUP_PHASE_ALREADY_COMPLETED
#[test]
fn test_complete_setup_phase_already_completed() {
    let mut state = MvxEsdtSafeTestState::new();

    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    state.complete_setup_phase(None, Some("unpauseContract"));
    state
        .common_setup
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc.is_setup_phase_complete());
        });

    state.complete_setup_phase(Some(SETUP_PHASE_ALREADY_COMPLETED), None);
}

/// ### TEST
/// M-ESDT_DEP_FAIL_011
///
/// ### ACTION
/// Call 'deposit()' with too many tokens in payments_vec
///
/// ### EXPECTED
/// Error TOO_MANY_TOKENS
#[test]
fn test_deposit_too_many_tokens() {
    let mut state = MvxEsdtSafeTestState::new();

    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);
    let esdt_token_payment = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::default(),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment; 11]);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        payments_vec,
        Some(TOO_MANY_TOKENS),
        None,
    );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::zero(),
    );
}

/// ### TEST
/// M-ESDT_DEP_OK_012
///
/// ### ACTION
/// Call 'deposit()' with valid payments_vec and no transfer_data
///
/// ### EXPECTED
/// * USER's balance is updated
#[test]
fn test_deposit_no_transfer_data() {
    let mut state = MvxEsdtSafeTestState::new();

    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        payments_vec,
        None,
        Some("deposit"),
    );

    let tokens_vec = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::from(100u64))),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(100u64))),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), tokens_vec);
}

/// ### TEST
/// M-ESDT_DEP_FAIL_013
///
/// ### ACTION
/// Call 'deposit()' with gas limit too high in transfer_data
///
/// ### EXPECTED
/// Error GAS_LIMIT_TOO_HIGH
#[test]
fn test_deposit_gas_limit_too_high() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        1,
        ManagedVec::new(),
        ManagedVec::new(),
    );
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.complete_setup_phase(None, None);
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);
    state.common_setup.deploy_testing_sc();

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec,
        Some(GAS_LIMIT_TOO_HIGH),
        None,
    );

    let tokens_vec = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::from(0u64))),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(0u64))),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), tokens_vec);
}

/// ### TEST
/// M-ESDT_DEP_FAIL_014
///
/// ### ACTION
/// Call 'deposit()' with max bridged amount exceeded
///
/// ### EXPECTED
/// Error DEPOSIT_OVER_MAX_AMOUNT
#[test]
fn test_deposit_max_bridged_amount_exceeded() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from("hello")]),
        ManagedVec::from(vec![MaxBridgedAmount {
            token_id: TokenIdentifier::from(FIRST_TEST_TOKEN),
            amount: BigUint::default(),
        }]),
    );

    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.complete_setup_phase(None, None);
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        payments_vec,
        Some(DEPOSIT_OVER_MAX_AMOUNT),
        None,
    );

    let tokens_vec = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::from(0u64))),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(0u64))),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), tokens_vec);
}

/// ### TEST
/// M-ESDT_DEP_FAIL_015
///
/// ### ACTION
/// Call 'deposit()' with banned endpoint name in transfer_data
///
/// ### EXPECTED
/// Error BANNED_ENDPOINT_NAME
#[test]
fn test_deposit_endpoint_banned() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from("hello")]),
        ManagedVec::new(),
    );

    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.complete_setup_phase(None, None);
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    let tokens_vec = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::from(0u64))),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(0u64))),
    ];

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec,
        Some(BANNED_ENDPOINT_NAME),
        None,
    );

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), tokens_vec);
}

/// ### TEST
/// M-ESDT_DEP_FAIL_016
///
/// ### ACTION
/// Call 'deposit()' with no transfer_data and no payments_vec
///
/// ### EXPECTED
/// Error NOTHING_TO_TRANSFER
#[test]
fn test_deposit_no_transfer_data_no_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::new(),
        Some(NOTHING_TO_TRANSFER),
        None,
    );
}

/// ### TEST
/// M-ESDT_DEP_OK_017
///
/// ### ACTION
/// Call 'deposit()' with transfer data only and no payments
///
/// ### EXPECTED
/// The endpoint is called in the testing smart contract
#[test]
fn test_deposit_transfer_data_only_no_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::Some(transfer_data),
        PaymentsVec::new(),
        None,
        Some("scCall"),
    );
}

/// ### TEST
/// M-ESDT_DEP_FAIL_018
///
/// ### ACTION
/// Call 'deposit()' with transfer data only, no payments and fee set
///
/// ### EXPECTED
/// Error ERR_EMPTY_PAYMENTS
#[test]
fn test_deposit_transfer_data_only_with_fee_nothing_to_transfer() {
    let mut state = MvxEsdtSafeTestState::new();

    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::Some(transfer_data),
        PaymentsVec::new(),
        Some(ERR_EMPTY_PAYMENTS),
        None,
    );
}

/// ### TEST
/// M-ESDT_DEP_OK_019
///
/// ### ACTION
/// Call 'deposit()' with transfer data and fee payment
///
/// ### EXPECTED
/// The endpoint is called in the testing smart contract
#[test]
fn test_deposit_transfer_data_only_with_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let payments_vec = PaymentsVec::from(fee_payment);

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec,
        None,
        Some("scCall"),
    );

    state.common_setup.check_account_single_esdt(
        FEE_MARKET_ADDRESS.to_address(),
        FEE_TOKEN,
        0u64,
        gas_limit.into(),
    );
}

/// ### TEST
/// M-ESDT_DEP_OK_020
///
/// ### ACTION
/// Call 'deposit()' with transfer data and valid payment
///
/// ### EXPECTED
/// USER's balance is updated
#[test]
fn test_deposit_fee_enabled() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.complete_setup_phase(None, None);

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec.clone(),
        None,
        Some("deposit"),
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    let expected_balances = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, expected_amount_token_one)),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, expected_amount_token_two)),
        MultiValue3::from((FEE_TOKEN, 0u64, expected_amount_token_fee)),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(OWNER_ADDRESS.to_address(), expected_balances);
}

/// ### TEST
/// M-ESDT_DEP_FAIL_021
///
/// ### ACTION
/// Call 'deposit()' with transfer data and payment not enough for fee
///
/// ### EXPECTED
/// Error PAYMENT_DOES_NOT_COVER_FEE
#[test]
fn test_deposit_payment_doesnt_cover_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.complete_setup_phase(None, None);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FIRST_TEST_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FIRST_TEST_TOKEN),
            per_transfer: BigUint::from(1u64),
            per_gas: BigUint::from(1u64),
        },
    };

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 10_000;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec,
        Some(PAYMENT_DOES_NOT_COVER_FEE),
        None,
    );

    let tokens_vec = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::from(0u64))),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(0u64))),
    ];
    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), tokens_vec);
}

/// ### TEST
/// M-ESDT_DEP_FAIL_022
///
/// ### ACTION
/// Call 'deposit()' with transfer data and non-whitelisted tokens
///
/// ### EXPECTED
/// The tokens are refunded back to the user, except the fee
#[test]
fn test_deposit_refund() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::from(vec![TokenIdentifier::from(CROWD_TOKEN_ID)]),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.complete_setup_phase(None, None);

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec.clone(),
        None,
        Some("deposit"),
    );

    let expected_balances = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::from(ONE_HUNDRED_MILLION))),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(ONE_HUNDRED_MILLION))),
        MultiValue3::from((
            FEE_TOKEN,
            0u64,
            BigUint::from(ONE_HUNDRED_MILLION - gas_limit as u32),
        )),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(OWNER_ADDRESS.to_address(), expected_balances);
}

/// ### TEST
/// M-ESDT_DEP_OK_023
///
/// ### ACTION
/// Call 'deposit()' with burn mechanism set
///
/// ### EXPECTED
/// USER's balance is updated
#[test]
fn test_deposit_success_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles();
    state.complete_setup_phase(None, None);
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    let esdt_token_payment_trusted_token = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        esdt_token_payment_trusted_token.clone(),
        esdt_token_payment_two.clone(),
    ]);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        payments_vec,
        None,
        Some("deposit"),
    );

    let expected_tokens = vec![
        MultiValue3::from((
            TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
            0u64,
            BigUint::from(0u64),
        )),
        MultiValue3::from((SECOND_TEST_TOKEN, 100u64, BigUint::from(0u64))),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), expected_tokens);

    let tokens = vec![
        (TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]), 100u64),
        (SECOND_TEST_TOKEN, 0u64),
    ];

    state.common_setup.check_deposited_tokens_amount(tokens);
}

/// ### TEST
/// M-ESDT_REG_OK_024
///
/// ### ACTION
/// Call 'register_token()' with valid token attributes
///
/// ### EXPECTED
/// The token is registered
#[test]
fn test_register_token_fungible_token_with_prefix() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = SOV_TOKEN;
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = FIRST_TEST_TOKEN.as_str();
    let num_decimals = 3;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, None);

    // TODO: add check for storage after callback fix
}

/// ### TEST
/// M-ESDT_REG_FAIL_025
///
/// ### ACTION
/// Call 'register_token()' with no prefix and type fungible
///
/// ### EXPECTED
/// Error CANNOT_REGISTER_TOKEN
#[test]
fn test_register_token_fungible_token_no_prefix() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = FIRST_TEST_TOKEN;
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = FIRST_TEST_TOKEN.as_str();
    let num_decimals = 3;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(
        register_token_args,
        egld_payment,
        Some(CANNOT_REGISTER_TOKEN),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(FIRST_TEST_TOKEN.as_str());
}

/// ### TEST
/// M-ESDT_REG_FAIL_026
///
/// ### ACTION
/// Call register_token twice
///
/// ### EXPECTED
/// The first token is registered and then error NATIVE_TOKEN_ALREADY_REGISTERED
#[test]
fn test_register_native_token_already_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(
        FIRST_TEST_TOKEN.as_str(),
        token_display_name,
        egld_payment.clone(),
        None,
    );

    // TODO: Add check for storage after callback issue is fixed

    state.register_native_token(
        FIRST_TEST_TOKEN.as_str(),
        token_display_name,
        egld_payment.clone(),
        None,
        // NOTE: Some(NATIVE_TOKEN_ALREADY_REGISTERED) when fix is here,
    );
}

/// ### TEST
/// M-ESDT_REG_OK_027
///
/// ### ACTION
/// Call 'register_native_token()' with valid token attributes
///
/// ### EXPECTED
/// The token is registered
#[test]
fn test_register_native_token() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(
        FIRST_TEST_TOKEN.as_str(),
        token_display_name,
        egld_payment.clone(),
        None,
    );

    // TODO: Check storage
}

/// ### TEST
/// M-ESDT_EXEC_FAIL_028
///
/// ### ACTION
/// Call 'execute_operation()' with no esdt-safe-address set
///
/// ### EXPECTED
/// Error NO_ESDT_SAFE_ADDRESS
#[test]
fn test_execute_operation_no_esdt_safe_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, config);
    state.complete_setup_phase(None, None);

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        EsdtTokenData::default(),
    );

    let operation_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let hash_of_hashes = state.common_setup.get_operation_hash(&operation);

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(NO_ESDT_SAFE_ADDRESS),
        None,
        None,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&hash_of_hashes);
}

/// ### TEST
/// M-ESDT_EXEC_OK_029
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[test]
fn test_execute_operation_success() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, config);

    state.complete_setup_phase(None, Some("unpauseContract"));

    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(TokenIdentifier::from(FIRST_TEST_TOKEN), 0, token_data);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data =
        OperationData::new(1, OWNER_ADDRESS.to_managed_address(), Some(transfer_data));

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);
    state.common_setup.deploy_testing_sc();
    state
        .common_setup
        .set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.common_setup.register_operation(
        CallerAddress::Owner,
        ManagedBuffer::new(),
        &hash_of_hashes,
        operations_hashes,
    );

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        None,
        Some("executedBridgeOp"),
        None,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}

/// ### TEST
/// M-ESDT_EXEC_OK_030
///
/// ### ACTION
/// Call 'execute_operation()' with payment containing the registered token
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[test]
fn test_execute_operation_with_native_token_success() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(
        FIRST_TEST_TOKEN.as_str(),
        token_display_name,
        egld_payment,
        None,
    );

    state.complete_setup_phase(None, Some("unpauseContract"));

    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(TokenIdentifier::from(FIRST_TEST_TOKEN), 0, token_data);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data =
        OperationData::new(1, OWNER_ADDRESS.to_managed_address(), Some(transfer_data));

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);
    state.common_setup.deploy_testing_sc();
    state
        .common_setup
        .set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.common_setup.register_operation(
        CallerAddress::Owner,
        ManagedBuffer::new(),
        &hash_of_hashes,
        operations_hashes,
    );

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        None,
        Some("executedBridgeOp"),
        None,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);

    state.common_setup.check_account_single_esdt(
        TESTING_SC_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
        0u64,
        BigUint::from(0u64),
    );
}

/// ### TEST
/// M-ESDT_EXEC_OK_031
///
/// ### ACTION
/// Call 'execute_operation()' after setting the burn mechanism
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[test]
fn test_execute_operation_burn_mechanism_without_deposit_cannot_subtract() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    let token_display_name = "NativeToken";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(TRUSTED_TOKEN_IDS[0], token_display_name, egld_payment, None);
    state.complete_setup_phase(None, Some("unpauseContract"));

    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment =
        OperationEsdtPayment::new(TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]), 0, token_data);

    let operation_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state.common_setup.deploy_testing_sc();
    state
        .common_setup
        .set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.common_setup.register_operation(
        CallerAddress::Owner,
        ManagedBuffer::new(),
        &hash_of_hashes,
        operations_hashes,
    );
    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        None,
        Some("executedBridgeOp"),
        None,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);

    state.common_setup.check_account_single_esdt(
        TESTING_SC_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
        0u64,
        BigUint::from(0u64),
    );
}

/// ### TEST
/// M-ESDT_EXEC_OK_032
///
/// ### ACTION
/// Call 'execute_operation()' after setting the burn mechanism
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[test]
fn test_execute_operation_success_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();
    state.complete_setup_phase(None, Some("unpauseContract"));

    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]),
        0,
        token_data.clone(),
    );

    let operation_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment.clone()].into(),
        operation_data,
    );

    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);
    state.common_setup.deploy_testing_sc();
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);
    state
        .common_setup
        .set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::from(vec![payment]),
        None,
        Some("deposit"),
    );

    state.common_setup.register_operation(
        CallerAddress::Owner,
        ManagedBuffer::new(),
        &hash_of_hashes,
        operations_hashes,
    );

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        None,
        Some("executedBridgeOp"),
        None,
    );

    let expected_amount_trusted_token = BigUint::from(ONE_HUNDRED_MILLION) - &token_data.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]),
            &expected_amount_trusted_token,
        );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
        0u64,
        BigUint::from(0u64),
    );

    state
        .common_setup
        .check_deposited_tokens_amount(vec![(TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]), 0)]);

    state.common_setup.check_account_single_esdt(
        TESTING_SC_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
        0u64,
        BigUint::from(100u64),
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}

/// ### TEST
/// M-ESDT_EXEC_OK_033
///
/// ### ACTION
/// Call 'execute_operation()' after switching to the lock mechanism from the burn mechanism
///
/// ### EXPECTED
/// The operation is executed the first time in the testing smart contract
/// The operation is executed again in the testing smart contract
#[test]
fn test_deposit_execute_switch_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();
    state.complete_setup_phase(None, Some("unpauseContract"));

    let trusted_token_id = TRUSTED_TOKEN_IDS[0];

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);
    state.common_setup.deploy_testing_sc();
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);
    state
        .common_setup
        .set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let deposited_trusted_token_payment_amount = 1000u64;
    let deposit_trusted_token_payment_token_data = EsdtTokenData {
        amount: BigUint::from(deposited_trusted_token_payment_amount),
        ..Default::default()
    };
    let deposit_trusted_token_payment = OperationEsdtPayment::new(
        TokenIdentifier::from(trusted_token_id),
        0,
        deposit_trusted_token_payment_token_data,
    );

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::from(vec![deposit_trusted_token_payment.clone()]),
        None,
        Some("deposit"),
    );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0,
        BigUint::from(1000u64),
    );

    state.set_token_burn_mechanism(trusted_token_id, None);

    let mut expected_deposited_amount = deposited_trusted_token_payment_amount;

    state.common_setup.check_deposited_tokens_amount(vec![(
        TestTokenIdentifier::new(trusted_token_id),
        expected_deposited_amount,
    )]);

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0u64,
        BigUint::from(0u64),
    );

    let execute_trusted_token_payment_amount = 500u64;
    let execute_trusted_token_payment_token_data = EsdtTokenData {
        amount: BigUint::from(execute_trusted_token_payment_amount),
        ..Default::default()
    };
    let execute_trusted_token_payment = OperationEsdtPayment::new(
        TokenIdentifier::from(trusted_token_id),
        0,
        execute_trusted_token_payment_token_data,
    );
    let operation_one_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);
    let operation_one = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![execute_trusted_token_payment.clone()].into(),
        operation_one_data,
    );
    let operation_one_hash = state.common_setup.get_operation_hash(&operation_one);
    let hash_of_hashes_one = ManagedBuffer::new_from_bytes(&sha256(&operation_one_hash.to_vec()));
    let operations_hashes_one =
        MultiValueEncoded::from(ManagedVec::from(vec![operation_one_hash.clone()]));

    state.common_setup.register_operation(
        CallerAddress::Owner,
        ManagedBuffer::new(),
        &hash_of_hashes_one,
        operations_hashes_one,
    );

    state.execute_operation(
        &hash_of_hashes_one,
        &operation_one,
        None,
        Some("executedBridgeOp"),
        None,
    );

    let mut expected_receiver_amount = execute_trusted_token_payment_amount;
    expected_deposited_amount -= execute_trusted_token_payment_amount;

    state.common_setup.check_deposited_tokens_amount(vec![(
        TestTokenIdentifier::new(trusted_token_id),
        expected_deposited_amount,
    )]);

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0u64,
        BigUint::from(0u64),
    );

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::from(vec![deposit_trusted_token_payment.clone()]),
        None,
        Some("deposit"),
    );

    expected_deposited_amount += deposited_trusted_token_payment_amount;

    state.common_setup.check_deposited_tokens_amount(vec![(
        TestTokenIdentifier::new(trusted_token_id),
        expected_deposited_amount,
    )]);

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0u64,
        BigUint::from(0u64),
    );

    state.set_token_lock_mechanism(trusted_token_id, None);

    state
        .common_setup
        .check_deposited_tokens_amount(vec![(TestTokenIdentifier::new(trusted_token_id), 0)]);

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0u64,
        BigUint::from(expected_deposited_amount),
    );

    let operation_two_data = OperationData::new(2, OWNER_ADDRESS.to_managed_address(), None);
    let operation_two = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![execute_trusted_token_payment.clone()].into(),
        operation_two_data,
    );
    let operation_two_hash = state.common_setup.get_operation_hash(&operation_two);
    let hash_of_hashes_two = ManagedBuffer::new_from_bytes(&sha256(&operation_two_hash.to_vec()));
    let operations_hashes_two =
        MultiValueEncoded::from(ManagedVec::from(vec![operation_two_hash.clone()]));

    state.common_setup.register_operation(
        CallerAddress::Owner,
        ManagedBuffer::new(),
        &hash_of_hashes_two,
        operations_hashes_two,
    );

    state.execute_operation(
        &hash_of_hashes_two,
        &operation_two,
        None,
        Some("executedBridgeOp"),
        None,
    );

    state
        .common_setup
        .check_deposited_tokens_amount(vec![(TestTokenIdentifier::new(trusted_token_id), 0)]);

    expected_receiver_amount += execute_trusted_token_payment_amount;
    expected_deposited_amount -= execute_trusted_token_payment_amount;

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0u64,
        BigUint::from(expected_deposited_amount),
    );

    state.common_setup.check_account_single_esdt(
        TESTING_SC_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0u64,
        BigUint::from(expected_receiver_amount),
    );

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::from(vec![deposit_trusted_token_payment]),
        None,
        Some("deposit"),
    );

    expected_deposited_amount += deposited_trusted_token_payment_amount;

    state
        .common_setup
        .check_deposited_tokens_amount(vec![(TestTokenIdentifier::new(trusted_token_id), 0)]);

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0u64,
        BigUint::from(expected_deposited_amount),
    );

    state.common_setup.check_account_single_esdt(
        TESTING_SC_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0u64,
        BigUint::from(expected_receiver_amount),
    );
}

/// ### TEST
/// M-ESDT_EXEC_OK_034
///
/// ### ACTION
/// Call 'execute_operation()' with empty payments
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[test]
fn test_execute_operation_no_payments() {
    let mut state = MvxEsdtSafeTestState::new();
    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(
        FIRST_TEST_TOKEN.as_str(),
        token_display_name,
        egld_payment,
        None,
    );

    state.complete_setup_phase(None, Some("unpauseContract"));

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data =
        OperationData::new(1, OWNER_ADDRESS.to_managed_address(), Some(transfer_data));

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        ManagedVec::new(),
        operation_data,
    );

    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.deploy_testing_sc();
    state
        .common_setup
        .set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.common_setup.register_operation(
        CallerAddress::Owner,
        ManagedBuffer::new(),
        &hash_of_hashes,
        operations_hashes,
    );

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        None,
        Some("executedBridgeOp"),
        None,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}

/// ### TEST
/// M-ESDT_EXEC_OK_035
///
/// ### ACTION
/// Call 'execute_operation()' with empty payments and wrong endpoint
///
/// ### EXPECTED
/// The operation is not executed in the testing smart contract
#[test]
fn test_execute_operation_no_payments_failed_event() {
    let mut state = MvxEsdtSafeTestState::new();
    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(
        FIRST_TEST_TOKEN.as_str(),
        token_display_name,
        egld_payment,
        None,
    );

    state.complete_setup_phase(None, None);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("WRONG_ENDPOINT");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data =
        OperationData::new(1, OWNER_ADDRESS.to_managed_address(), Some(transfer_data));

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        ManagedVec::new(),
        operation_data,
    );

    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state.common_setup.deploy_testing_sc();
    state
        .common_setup
        .set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.common_setup.register_operation(
        CallerAddress::Owner,
        ManagedBuffer::new(),
        &hash_of_hashes,
        operations_hashes,
    );

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        None,
        Some("executedBridgeOp"),
        Some("invalid function (not found)"),
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}

/// ### TEST
/// M-ESDT_SET_BURN_FAIL_036
///
/// ### ACTION
/// Call 'set_token_burn_mechanism()' without the propper roles
///
/// ### EXPECTED
/// Error MINT_AND_BURN_ROLES_NOT_FOUND
#[test]
fn test_set_token_burn_mechanism_no_roles() {
    let mut state = MvxEsdtSafeTestState::new();
    state.common_setup.deploy_mvx_esdt_safe(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    state.set_token_burn_mechanism("WEGLD", Some(MINT_AND_BURN_ROLES_NOT_FOUND));
}

/// ### TEST
/// M-ESDT_SET_BURN_FAIL_037
///
/// ### ACTION
/// Call 'set_token_burn_mechanism()' without a trusted token id
///
/// ### EXPECTED
/// Error TOKEN_ID_IS_NOT_TRUSTED
#[test]
fn test_set_token_burn_mechanism_token_not_trusted() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(FIRST_TEST_TOKEN.as_str(), Some(TOKEN_ID_IS_NOT_TRUSTED));
}

/// ### TEST
/// M-ESDT_SET_BURN_OK_038
///
/// ### ACTION
/// Call 'set_token_burn_mechanism()' with a trusted token id
///
/// ### EXPECTED
/// The trusted token has the burn mechanism set
#[test]
fn test_set_token_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state
        .common_setup
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc
                .burn_mechanism_tokens()
                .contains(&TokenIdentifier::from(TRUSTED_TOKEN_IDS[0])))
        });

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
        0u64,
        BigUint::from(0u64),
    );
}

/// ### TEST
/// M-ESDT_SET_BURN_OK_039
///
/// ### ACTION
/// Call both 'set_token_burn_mechanism()' and 'set_token_lock_mechanism()' with a trusted token id.
///
/// ### EXPECTED
/// The trusted token has the lock mechanism set
#[test]
fn test_set_token_lock_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);
    state.set_token_lock_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state
        .common_setup
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc.burn_mechanism_tokens().is_empty())
        });

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
        100u64,
        BigUint::from(0u64),
    );
}

/// ### TEST
/// M-ESDT_SET_BURN_FAIL_040
///
/// ### ACTION
/// Call both 'set_token_burn_mechanism()' and 'set_token_lock_mechanism()' with a trusted token id.
///
/// ### EXPECTED
/// ERROR TOKEN_IS_FROM_SOVEREIGN
#[test]
fn test_set_token_lock_mechanism_token_from_sovereign() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            sc.multiversx_to_sovereign_token_id_mapper(&TokenIdentifier::from(
                TRUSTED_TOKEN_IDS[0],
            ))
            .set(TokenIdentifier::from("MOCK"));
        });

    state.set_token_lock_mechanism(TRUSTED_TOKEN_IDS[0], Some(TOKEN_IS_FROM_SOVEREIGN));
}
