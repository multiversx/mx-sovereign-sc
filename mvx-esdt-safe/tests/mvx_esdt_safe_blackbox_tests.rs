use common_test_setup::constants::{
    CROWD_TOKEN_ID, DEPOSIT_EVENT, ESDT_SAFE_ADDRESS, EXECUTED_BRIDGE_OP_EVENT, FEE_MARKET_ADDRESS,
    FEE_TOKEN, FIRST_TEST_TOKEN, FIRST_TOKEN_ID, HEADER_VERIFIER_ADDRESS, ISSUE_COST,
    NATIVE_TEST_TOKEN, ONE_HUNDRED_MILLION, ONE_HUNDRED_THOUSAND, ONE_HUNDRED_TOKENS,
    OWNER_ADDRESS, PER_GAS, PER_TRANSFER, SC_CALL_EVENT, SECOND_TEST_TOKEN, SECOND_TOKEN_ID,
    SOV_FIRST_TOKEN_ID, SOV_SECOND_TOKEN_ID, SOV_TOKEN, TESTING_SC_ADDRESS, TESTING_SC_ENDPOINT,
    TRUSTED_TOKEN, UNPAUSE_CONTRACT_LOG, USER_ADDRESS, WRONG_ENDPOINT_NAME,
};
use cross_chain::storage::CrossChainStorage;
use cross_chain::{DEFAULT_ISSUE_COST, MAX_GAS_PER_TRANSACTION};
use error_messages::{
    BANNED_ENDPOINT_NAME, CALLER_IS_BLACKLISTED, CALLER_NOT_FROM_CURRENT_SOVEREIGN,
    CURRENT_OPERATION_NOT_REGISTERED, DEPOSIT_OVER_MAX_AMOUNT, ERR_EMPTY_PAYMENTS,
    GAS_LIMIT_TOO_HIGH, INVALID_FUNCTION_NOT_FOUND, INVALID_PREFIX_FOR_REGISTER, INVALID_TYPE,
    MAX_GAS_LIMIT_PER_TX_EXCEEDED, MINT_AND_BURN_ROLES_NOT_FOUND, NATIVE_TOKEN_ALREADY_REGISTERED,
    NATIVE_TOKEN_NOT_REGISTERED, NOTHING_TO_TRANSFER, NOT_ENOUGH_EGLD_FOR_REGISTER,
    PAYMENT_DOES_NOT_COVER_FEE, SETUP_PHASE_NOT_COMPLETED, TOKEN_ID_IS_NOT_TRUSTED,
    TOO_MANY_TOKENS,
};
use header_verifier::storage::HeaderVerifierStorageModule;
use multiversx_sc::chain_core::EGLD_000000_TOKEN_IDENTIFIER;
use multiversx_sc::types::{
    EgldOrEsdtTokenIdentifier, EgldOrEsdtTokenPayment, MultiEgldOrEsdtPayment, MultiValueEncoded,
    ReturnsHandledOrError,
};
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EsdtTokenData, EsdtTokenPayment, EsdtTokenType, ManagedBuffer, ManagedVec,
        TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_scenario::ScenarioTxRun;
use multiversx_sc_scenario::{api::StaticApi, ScenarioTxWhitebox};
use mvx_esdt_safe::bridging_mechanism::BridgingMechanism;
use mvx_esdt_safe_blackbox_setup::MvxEsdtSafeTestState;
use proxies::mvx_esdt_safe_proxy::MvxEsdtSafeProxy;
use setup_phase::SetupPhaseModule;
use structs::aliases::OptionalValueTransferDataTuple;
use structs::configs::{
    MaxBridgedAmount, SetBurnMechanismOperation, SetLockMechanismOperation, SovereignConfig,
    UpdateEsdtSafeConfigOperation,
};
use structs::fee::{FeeStruct, FeeType};
use structs::forge::ScArray;
use structs::generate_hash::GenerateHash;
use structs::operation::TransferData;
use structs::{
    aliases::PaymentsVec,
    configs::EsdtSafeConfig,
    operation::{Operation, OperationData, OperationEsdtPayment},
};
use structs::{OperationHashStatus, RegisterTokenOperation};
mod mvx_esdt_safe_blackbox_setup;

/// ### TEST
/// M-ESDT_DEPLOY_OK
///
/// ### ACTION
/// Call 'deploy_mvx_esdt_safe()' with default config
///
/// ### EXPECTED
/// Contract is deployed with the default config
#[test]
fn test_deploy() {
    let mut state = MvxEsdtSafeTestState::new();

    state
        .common_setup
        .deploy_mvx_esdt_safe(OptionalValue::Some(EsdtSafeConfig::default_config()));
}

/// ### TEST
/// M-ESDT_DEPLOY_FAIL
///
/// ### ACTION
/// Call 'update_configuration()' with invalid config
///
/// ### EXPECTED
/// Error MAX_GAS_LIMIT_PER_TX_EXCEEDED
#[test]
fn test_update_invalid_config() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);

    let config = EsdtSafeConfig {
        max_tx_gas_limit: MAX_GAS_PER_TRANSACTION + 1,
        ..EsdtSafeConfig::default_config()
    };

    state.update_esdt_safe_config_during_setup_phase(config, Some(MAX_GAS_LIMIT_PER_TX_EXCEEDED));
}

/// ### TEST
/// M-ESDT_REG_FAIL
///
/// ### ACTION
/// Call 'register_token()' with invalid token type
///
/// ### EXPECTED
/// Error CANNOT_REGISTER_TOKEN
#[test]
#[ignore = "needs blackbox callback fix"]
fn test_register_token_invalid_type() {
    let mut state = MvxEsdtSafeTestState::new();

    let sov_token_id = format!("sov-{}", FIRST_TEST_TOKEN.as_str());
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = FIRST_TEST_TOKEN.as_str();

    let register_token_args = RegisterTokenOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(sov_token_id.as_str()),
        token_type,
        token_display_name: token_display_name.into(),
        token_ticker: token_ticker.into(),
        num_decimals,
        data: OperationData::new(0u64, USER_ADDRESS.to_managed_address(), None),
    };

    let token_hash = register_token_args.generate_hash();
    let hash_of_hashes = ManagedBuffer::from(&sha256(&token_hash.to_vec()));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    let payment =
        EgldOrEsdtTokenPayment::new(EGLD_000000_TOKEN_IDENTIFIER.into(), 0u64, ISSUE_COST.into());

    let signature = state.deploy_and_complete_setup_phase(&hash_of_hashes);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        ManagedVec::from_single_item(payment),
        None,
        Some(DEPOSIT_EVENT),
    );

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![token_hash]),
    );

    state.register_token(
        register_token_args,
        hash_of_hashes,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(INVALID_TYPE),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(SECOND_TEST_TOKEN.as_str());
}

/// ### TEST
/// M-ESDT_REG_FAIL
///
/// ### ACTION
/// Call 'register_token()' with invalid token type and prefix
///
/// ### EXPECTED
/// Error INVALID_TYPE
#[test]
#[ignore = "needs blackbox callback fix"]
fn test_register_token_invalid_type_with_prefix() {
    let mut state = MvxEsdtSafeTestState::new();

    let sov_token_id = SOV_TOKEN;
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = FIRST_TEST_TOKEN.as_str();

    let register_token_args = RegisterTokenOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(sov_token_id),
        token_type,
        token_display_name: token_display_name.into(),
        token_ticker: token_ticker.into(),
        num_decimals,
        data: OperationData::new(0u64, USER_ADDRESS.to_managed_address(), None),
    };

    let token_hash = register_token_args.generate_hash();
    let hash_of_hashes = ManagedBuffer::from(&sha256(&token_hash.to_vec()));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    let payment =
        EgldOrEsdtTokenPayment::new(EGLD_000000_TOKEN_IDENTIFIER.into(), 0u64, ISSUE_COST.into());

    let signature = state.deploy_and_complete_setup_phase(&hash_of_hashes);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        ManagedVec::from_single_item(payment),
        None,
        Some(DEPOSIT_EVENT),
    );

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![token_hash]),
    );

    state.register_token(
        register_token_args,
        hash_of_hashes,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(INVALID_TYPE),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(SECOND_TEST_TOKEN.as_str());
}

/// ### TEST
/// M-ESDT_REG_FAIL
///
/// ### ACTION
/// Call 'register_token()' with token id not starting with prefix and not enough egld in balance
///
/// ### EXPECTED
/// Error CANNOT_REGISTER_TOKEN
#[test]
fn test_register_token_not_enough_egld() {
    let mut state = MvxEsdtSafeTestState::new();

    let sov_token_id = SECOND_TEST_TOKEN;
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = FIRST_TEST_TOKEN.as_str();

    let register_token_args = RegisterTokenOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(sov_token_id),
        token_type,
        token_display_name: token_display_name.into(),
        token_ticker: token_ticker.into(),
        num_decimals,
        data: OperationData::new(0u64, USER_ADDRESS.to_managed_address(), None),
    };

    let token_hash = register_token_args.generate_hash();
    let hash_of_hashes = ManagedBuffer::from(&sha256(&token_hash.to_vec()));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    let signature = state.deploy_and_complete_setup_phase(&hash_of_hashes);

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![token_hash]),
    );

    state.register_token(
        register_token_args,
        hash_of_hashes,
        Some(DEPOSIT_EVENT),
        Some(NOT_ENOUGH_EGLD_FOR_REGISTER),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(SECOND_TEST_TOKEN.as_str());
}

/// ### TEST
/// M-ESDT_REG_OK
///
/// ### ACTION
/// Call 'register_token()' with valid token id and type
///
/// ### EXPECTED
/// The token is registered
#[test]
fn test_register_token_fungible_token() {
    let mut state = MvxEsdtSafeTestState::new();

    let sov_token_id = SOV_TOKEN;
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = FIRST_TEST_TOKEN.as_str();
    let num_decimals = 3;

    let register_token_args = RegisterTokenOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(sov_token_id),
        token_type,
        token_display_name: token_display_name.into(),
        token_ticker: token_ticker.into(),
        num_decimals,
        data: OperationData::new(0u64, USER_ADDRESS.to_managed_address(), None),
    };

    let token_hash = register_token_args.generate_hash();
    let hash_of_hashes = ManagedBuffer::from(&sha256(&token_hash.to_vec()));
    let bitmap = state.common_setup.full_bitmap(1);

    let epoch = 0;

    let payment = EgldOrEsdtTokenPayment::egld_payment(ISSUE_COST.into());

    let signature = state.deploy_and_complete_setup_phase(&hash_of_hashes);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        ManagedVec::from_single_item(payment),
        None,
        Some(DEPOSIT_EVENT),
    );

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![token_hash.clone()]),
    );

    state.register_token(register_token_args, hash_of_hashes, Some(""), None);

    // TODO: add check for storage after callback fix
}

/// ### TEST
/// M-ESDT_REG_FAIL
///
/// ### ACTION
/// Call 'register_token()' with token id not starting with prefix and token type NonFungible
///
/// ### EXPECTED
/// Error CANNOT_REGISTER_TOKEN
#[test]
fn test_register_token_nonfungible_token() {
    let mut state = MvxEsdtSafeTestState::new();

    let sov_token_id = FIRST_TEST_TOKEN;
    let token_type = EsdtTokenType::NonFungible;
    let token_display_name = "TokenOne";
    let num_decimals = 0;
    let token_ticker = FIRST_TEST_TOKEN.as_str();

    let register_token_args = RegisterTokenOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(sov_token_id),
        token_type,
        token_display_name: token_display_name.into(),
        token_ticker: token_ticker.into(),
        num_decimals,
        data: OperationData::new(0u64, USER_ADDRESS.to_managed_address(), None),
    };

    let token_hash = register_token_args.generate_hash();
    let hash_of_hashes = ManagedBuffer::from(&sha256(&token_hash.to_vec()));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    let signature = state.deploy_and_complete_setup_phase(&hash_of_hashes);

    let payment =
        EgldOrEsdtTokenPayment::new(EGLD_000000_TOKEN_IDENTIFIER.into(), 0u64, ISSUE_COST.into());

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        ManagedVec::from_single_item(payment),
        None,
        Some(DEPOSIT_EVENT),
    );

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![token_hash]),
    );

    state.register_token(
        register_token_args,
        hash_of_hashes,
        Some(DEPOSIT_EVENT),
        Some(INVALID_PREFIX_FOR_REGISTER),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(SECOND_TEST_TOKEN.as_str());
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with empty payments_vec and no transfer_data
///
/// ### EXPECTED
/// Error NOTHING_TO_TRANSFER
#[test]
fn test_deposit_nothing_to_transfer() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

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
/// M-ESDT_DEP_OK
///
/// ### ACTION
/// Call 'deposit()' as a blacklisted caller and then remove from blacklist
///
/// ### EXPECTED
/// Error CALLER_IS_BLACKLISTED for the deposit attempt and successful removal from blacklist
#[test]
fn test_deposit_blacklist() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));
    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ESDTSafe]);
    let payment = EgldOrEsdtTokenPayment::egld_payment(ONE_HUNDRED_TOKENS.into());
    let payments_vec = PaymentsVec::from_single_item(payment);
    let caller = &USER_ADDRESS.to_managed_address();

    state.add_caller_to_deposit_blacklist(caller);

    let result = state
        .common_setup
        .world
        .tx()
        .from(USER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(MvxEsdtSafeProxy)
        .deposit(OWNER_ADDRESS, OptionalValueTransferDataTuple::None)
        .payment(payments_vec)
        .returns(ReturnsHandledOrError::new())
        .run();

    state
        .common_setup
        .assert_expected_error_message(result, Some(CALLER_IS_BLACKLISTED));

    state.remove_caller_from_deposit_blacklist(caller);
}

/// ### TEST
/// M-ESDT_SETUP_OK
///
/// ### ACTION
/// Call 'complete_setup_phase()'
///
/// ### EXPECTED
/// The setup phase is marked as completed in the smart contract's storage
#[test]
fn test_complete_setup_phase() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

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
/// M-ESDT_SETUP_FAIL
///
/// ### ACTION
/// Call 'complete_setup_phase()' twice
///
/// ### EXPECTED
/// Error SETUP_PHASE_ALREADY_COMPLETED
#[test]
fn test_complete_setup_phase_already_completed() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));
    state
        .common_setup
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc.is_setup_phase_complete());
        });

    state.complete_setup_phase_as_header_verifier(None, None);
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with too many tokens in payments_vec
///
/// ### EXPECTED
/// Error TOO_MANY_TOKENS
#[test]
fn test_deposit_too_many_tokens() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

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
/// M-ESDT_DEP_OK
///
/// ### ACTION
/// Call 'deposit()' with valid payments_vec and no transfer_data
///
/// ### EXPECTED
/// USER's balance is updated
#[test]
fn test_deposit_no_transfer_data() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

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

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        payments_vec,
        None,
        Some(DEPOSIT_EVENT),
    );

    let owner_tokens_vec = vec![
        MultiValue3::from((
            FIRST_TEST_TOKEN,
            0u64,
            BigUint::from(ONE_HUNDRED_MILLION - ONE_HUNDRED_THOUSAND),
        )),
        MultiValue3::from((
            SECOND_TEST_TOKEN,
            0u64,
            BigUint::from(ONE_HUNDRED_MILLION - ONE_HUNDRED_THOUSAND),
        )),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(OWNER_ADDRESS.to_address(), owner_tokens_vec);

    let tokens_vec = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::from(ONE_HUNDRED_THOUSAND))),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(ONE_HUNDRED_THOUSAND))),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), tokens_vec);
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with gas limit too high in transfer_data
///
/// ### EXPECTED
/// Error GAS_LIMIT_TOO_HIGH
#[test]
fn test_deposit_gas_limit_too_high() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig {
        max_tx_gas_limit: 1,
        ..EsdtSafeConfig::default_config()
    };
    state
        .common_setup
        .deploy_mvx_esdt_safe(OptionalValue::Some(config));

    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            sc.native_token()
                .set(EgldOrEsdtTokenIdentifier::esdt(SECOND_TEST_TOKEN));
        });

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

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

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
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
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::zero())),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::zero())),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), tokens_vec);
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with max bridged amount exceeded
///
/// ### EXPECTED
/// Error DEPOSIT_OVER_MAX_AMOUNT
#[test]
fn test_deposit_max_bridged_amount_exceeded() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig {
        max_bridged_token_amounts: ManagedVec::from(vec![MaxBridgedAmount {
            token_id: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
            amount: BigUint::default(),
        }]),
        ..EsdtSafeConfig::default_config()
    };

    state
        .common_setup
        .deploy_mvx_esdt_safe(OptionalValue::Some(config));
    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            sc.native_token()
                .set(EgldOrEsdtTokenIdentifier::esdt(SECOND_TEST_TOKEN));
        });

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

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

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        payments_vec,
        Some(DEPOSIT_OVER_MAX_AMOUNT),
        None,
    );

    let tokens_vec = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::zero())),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::zero())),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), tokens_vec);
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with banned endpoint name in transfer_data
///
/// ### EXPECTED
/// Error BANNED_ENDPOINT_NAME
#[test]
fn test_deposit_endpoint_banned() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig {
        banned_endpoints: ManagedVec::from(vec![ManagedBuffer::from(TESTING_SC_ENDPOINT)]),
        ..EsdtSafeConfig::default_config()
    };

    state
        .common_setup
        .deploy_mvx_esdt_safe(OptionalValue::Some(config));
    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            sc.native_token()
                .set(EgldOrEsdtTokenIdentifier::esdt(SECOND_TEST_TOKEN));
        });

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

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

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    let tokens_vec = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::zero())),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::zero())),
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
/// M-ESDT_DEP_OK
///
/// ### ACTION
/// Call 'deposit()' with transfer data only and no payments
///
/// ### EXPECTED
/// The endpoint is called in the testing smart contract
#[test]
fn test_deposit_transfer_data_only_no_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state.common_setup.deploy_testing_sc();

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
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
        Some(SC_CALL_EVENT),
    );
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with transfer data only, no payments and fee set
///
/// ### EXPECTED
/// Error ERR_EMPTY_PAYMENTS
#[test]
fn test_deposit_transfer_data_only_with_fee_nothing_to_transfer() {
    let mut state = MvxEsdtSafeTestState::new();

    let fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(FEE_TOKEN),
            per_transfer: PER_TRANSFER.into(),
            per_gas: PER_GAS.into(),
        },
    };

    state.deploy_contract_with_roles(Some(fee));
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state.common_setup.deploy_testing_sc();

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
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
/// M-ESDT_DEP_OK
///
/// ### ACTION
/// Call 'deposit()' with transfer data and fee payment
///
/// ### EXPECTED
/// The endpoint is called in the testing smart contract
#[test]
fn test_deposit_transfer_data_only_with_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    let fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(FEE_TOKEN),
            per_transfer: PER_TRANSFER.into(),
            per_gas: PER_GAS.into(),
        },
    };

    state.deploy_contract_with_roles(Some(fee));
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment = EgldOrEsdtTokenPayment::<StaticApi>::new(
        EgldOrEsdtTokenIdentifier::esdt(FEE_TOKEN),
        0,
        fee_amount.clone(),
    );

    let payments_vec = PaymentsVec::from(vec![fee_payment]);

    state.common_setup.deploy_testing_sc();

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
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
        Some(SC_CALL_EVENT),
    );

    state.common_setup.check_account_single_esdt(
        FEE_MARKET_ADDRESS.to_address(),
        FEE_TOKEN,
        0u64,
        gas_limit.into(),
    );
}

/// ### TEST
/// M-ESDT_DEP_OK
///
/// ### ACTION
/// Call 'deposit()' with transfer data and valid payment
///
/// ### EXPECTED
/// USER's balance is updated
#[test]
fn test_deposit_fee_enabled() {
    let mut state = MvxEsdtSafeTestState::new();

    let fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(FEE_TOKEN),
            per_transfer: PER_TRANSFER.into(),
            per_gas: PER_GAS.into(),
        },
    };

    state.deploy_contract_with_roles(Some(fee));
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state.common_setup.deploy_testing_sc();

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

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
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
        Some(DEPOSIT_EVENT),
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * PER_TRANSFER
        - BigUint::from(gas_limit) * PER_GAS;

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
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with transfer data and payment not enough for fee
///
/// ### EXPECTED
/// Error PAYMENT_DOES_NOT_COVER_FEE
#[test]
fn test_deposit_payment_doesnt_cover_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    let fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
            per_transfer: BigUint::from(PER_TRANSFER),
            per_gas: BigUint::from(PER_GAS),
        },
    };

    state.deploy_contract_with_roles(Some(fee));
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state.common_setup.deploy_testing_sc();

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(10u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 10_000;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
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
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::zero())),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::zero())),
    ];
    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), tokens_vec);
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with transfer data and non-whitelisted tokens
///
/// ### EXPECTED
/// The tokens are refunded back to the user, except the fee
#[test]
fn test_deposit_refund() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig {
        token_whitelist: ManagedVec::from(vec![EgldOrEsdtTokenIdentifier::esdt(CROWD_TOKEN_ID)]),
        ..EsdtSafeConfig::default_config()
    };

    state
        .common_setup
        .deploy_mvx_esdt_safe(OptionalValue::Some(config));

    let fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(FEE_TOKEN),
            per_transfer: PER_TRANSFER.into(),
            per_gas: PER_GAS.into(),
        },
    };

    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            sc.native_token()
                .set(EgldOrEsdtTokenIdentifier::esdt(SECOND_TEST_TOKEN));
        });

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state.common_setup.deploy_testing_sc();

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
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
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
        Some(DEPOSIT_EVENT),
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
/// M-ESDT_DEP_OK
///
/// ### ACTION
/// Call 'deposit()' with burn mechanism set
///
/// ### EXPECTED
/// USER's balance is updated
#[test]
fn test_deposit_success_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);
    state.set_token_burn_mechanism_before_setup_phase(TRUSTED_TOKEN, None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    let esdt_token_payment_trusted_token = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TRUSTED_TOKEN),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
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
        Some(DEPOSIT_EVENT),
    );

    let expected_tokens = vec![
        MultiValue3::from((
            TestTokenIdentifier::new(TRUSTED_TOKEN),
            0u64,
            BigUint::zero(),
        )),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(ONE_HUNDRED_THOUSAND))),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), expected_tokens);

    let tokens = vec![
        (
            EgldOrEsdtTokenIdentifier::esdt(TRUSTED_TOKEN),
            ONE_HUNDRED_THOUSAND.into(),
        ),
        (EgldOrEsdtTokenIdentifier::esdt(SECOND_TEST_TOKEN), 0u64),
    ];

    state.common_setup.check_deposited_tokens_amount(tokens);
}

/// ### TEST
/// M-ESDT_REG_OK
///
/// ### ACTION
/// Call 'register_token()' with valid token attributes
///
/// ### EXPECTED
/// The token is registered
#[test]
fn test_register_token_fungible_token_with_prefix() {
    let mut state = MvxEsdtSafeTestState::new();

    let sov_token_id = SOV_TOKEN;
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = FIRST_TEST_TOKEN.as_str();
    let num_decimals = 3;

    let register_token_args = RegisterTokenOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(sov_token_id),
        token_type,
        token_display_name: token_display_name.into(),
        token_ticker: token_ticker.into(),
        num_decimals,
        data: OperationData::new(0u64, USER_ADDRESS.to_managed_address(), None),
    };

    let token_hash = register_token_args.generate_hash();
    let hash_of_hashes = ManagedBuffer::from(&sha256(&token_hash.to_vec()));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    let payment =
        EgldOrEsdtTokenPayment::new(EGLD_000000_TOKEN_IDENTIFIER.into(), 0u64, ISSUE_COST.into());

    let signature = state.deploy_and_complete_setup_phase(&hash_of_hashes);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        ManagedVec::from_single_item(payment),
        None,
        Some(DEPOSIT_EVENT),
    );

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![token_hash]),
    );

    state.register_token(register_token_args, hash_of_hashes, Some(""), None);

    // TODO: add check for storage after callback fix
}

/// ### TEST
/// M-ESDT_REG_FAIL
///
/// ### ACTION
/// Call 'register_token()' with no prefix and type fungible
///
/// ### EXPECTED
/// Error CANNOT_REGISTER_TOKEN
#[test]
fn test_register_token_fungible_token_no_prefix() {
    let mut state = MvxEsdtSafeTestState::new();

    let sov_token_id = FIRST_TEST_TOKEN;
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = FIRST_TEST_TOKEN.as_str();
    let num_decimals = 3;

    let register_token_args = RegisterTokenOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(sov_token_id),
        token_type,
        token_display_name: token_display_name.into(),
        token_ticker: token_ticker.into(),
        num_decimals,
        data: OperationData::new(0u64, USER_ADDRESS.to_managed_address(), None),
    };

    let token_hash = register_token_args.generate_hash();
    let hash_of_hashes = ManagedBuffer::from(&sha256(&token_hash.to_vec()));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    let payment =
        EgldOrEsdtTokenPayment::new(EGLD_000000_TOKEN_IDENTIFIER.into(), 0u64, ISSUE_COST.into());

    let signature = state.deploy_and_complete_setup_phase(&hash_of_hashes);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        ManagedVec::from_single_item(payment),
        None,
        Some(DEPOSIT_EVENT),
    );

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![token_hash]),
    );

    state.register_token(
        register_token_args,
        hash_of_hashes,
        Some(DEPOSIT_EVENT),
        Some(INVALID_PREFIX_FOR_REGISTER),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(FIRST_TEST_TOKEN.as_str());
}

/// ### TEST
/// M-ESDT_REG_OK
///
/// ### ACTION
/// Call 'register_token()' with valid token attributes and token type DynamicNFT
///
/// ### EXPECTED
/// The token is registered
#[ignore = "Needs system sc function fix (registerAndSetAllRolesDynamic)"]
#[test]
fn test_register_token_non_fungible_token_dynamic() {
    let mut state = MvxEsdtSafeTestState::new();

    let sov_token_id = SOV_TOKEN;
    let token_type = EsdtTokenType::DynamicNFT;
    let token_display_name = "TokenOne";
    let token_ticker = FIRST_TEST_TOKEN.as_str();
    let num_decimals = 3;

    let register_token_args = RegisterTokenOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(sov_token_id),
        token_type,
        token_display_name: token_display_name.into(),
        token_ticker: token_ticker.into(),
        num_decimals,
        data: OperationData::new(0u64, USER_ADDRESS.to_managed_address(), None),
    };

    let token_hash = register_token_args.generate_hash();
    let hash_of_hashes = ManagedBuffer::from(&sha256(&token_hash.to_vec()));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    let payment =
        EgldOrEsdtTokenPayment::new(EGLD_000000_TOKEN_IDENTIFIER.into(), 0u64, ISSUE_COST.into());

    let signature = state.deploy_and_complete_setup_phase(&hash_of_hashes);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        ManagedVec::from_single_item(payment),
        None,
        Some(DEPOSIT_EVENT),
    );

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![token_hash]),
    );

    state.register_token(register_token_args, hash_of_hashes, Some(""), None);
}

/// ### TEST
/// M-ESDT_REG_FAIL
///
/// ### ACTION
/// Call register_token twice
///
/// ### EXPECTED
/// The first token is registered and then error NATIVE_TOKEN_ALREADY_REGISTERED
#[test]
fn test_register_native_token_already_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    // TODO: Add check for storage after callback issue is fixed

    state.register_native_token(
        FIRST_TEST_TOKEN.as_str(),
        token_display_name,
        egld_payment.clone(),
        Some(NATIVE_TOKEN_ALREADY_REGISTERED),
    );
}

/// ### TEST
/// M-ESDT_REG_FAIL
///
/// ### ACTION
/// Call complete_setup_phase() with no native token
///
/// ### EXPECTED
/// NATIVE_TOKEN_NOT_REGISTERED
#[test]
fn test_complete_setup_with_no_native_token() {
    let mut state = MvxEsdtSafeTestState::new();
    state.common_setup.deploy_mvx_esdt_safe(OptionalValue::None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    assert!(state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(MvxEsdtSafeProxy)
        .complete_setup_phase()
        .returns(ReturnsHandledOrError::new())
        .run()
        .is_err_and(|e| e.message == NATIVE_TOKEN_NOT_REGISTERED),);
}

/// ### TEST
/// M-ESDT_EXEC_FAIL
///
/// ### ACTION
/// Call 'execute_operation()' with no chain-config registered
///
/// ### EXPECTED
/// Error CALLER_NOT_FROM_CURRENT_SOVEREIGN
#[test]
fn test_execute_operation_no_chain_config_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    let payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        0,
        EsdtTokenData::default(),
    );

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            OWNER_ADDRESS.to_managed_address(),
            None,
        ),
    );

    let hash_of_hashes = state.common_setup.get_operation_hash(&operation);

    state.common_setup.deploy_header_verifier(vec![]);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        Some(CALLER_NOT_FROM_CURRENT_SOVEREIGN),
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&hash_of_hashes);
}

/// ### TEST
/// M-ESDT_EXEC_FAIL
///
/// ### ACTION
/// Call 'execute_operation()' with no esdt-safe-address set
///
/// ### EXPECTED
/// Error CALLER_NOT_FROM_CURRENT_SOVEREIGN
#[test]
fn test_execute_operation_no_esdt_safe_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    let payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        0,
        EsdtTokenData::default(),
    );

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            OWNER_ADDRESS.to_managed_address(),
            None,
        ),
    );

    let hash_of_hashes = state.common_setup.get_operation_hash(&operation);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig]);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        Some(CALLER_NOT_FROM_CURRENT_SOVEREIGN),
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&hash_of_hashes);
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[test]
fn test_execute_operation_success() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    let token_data = EsdtTokenData {
        amount: BigUint::from(ONE_HUNDRED_THOUSAND),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        0,
        token_data,
    );

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            OWNER_ADDRESS.to_managed_address(),
            Some(transfer_data),
        ),
    );

    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    state.common_setup.deploy_chain_config(
        OptionalValue::Some(SovereignConfig::default_config_for_test()),
        None,
    );

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );
    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);
    state.common_setup.deploy_testing_sc();

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        operations_hashes,
    );

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        None,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with payment containing the registered token
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[test]
fn test_execute_operation_with_native_token_success() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    let token_data = EsdtTokenData {
        amount: BigUint::from(ONE_HUNDRED_THOUSAND),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(NATIVE_TEST_TOKEN),
        0,
        token_data,
    );

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            OWNER_ADDRESS.to_managed_address(),
            Some(transfer_data),
        ),
    );

    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    state.common_setup.deploy_chain_config(
        OptionalValue::Some(SovereignConfig::default_config_for_test()),
        None,
    );

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);
    state.common_setup.deploy_testing_sc();

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        operations_hashes,
    );

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        None,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);

    state.common_setup.check_account_single_esdt(
        TESTING_SC_ADDRESS.to_address(),
        NATIVE_TEST_TOKEN,
        0u64,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' after setting the burn mechanism without prior deposit
///
/// ### EXPECTED
/// The operation executes successfully with minted tokens
#[test]
fn test_execute_operation_burn_mechanism_without_deposit_cannot_subtract() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    let payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(TRUSTED_TOKEN),
        0,
        EsdtTokenData {
            amount: BigUint::from(ONE_HUNDRED_THOUSAND),
            ..Default::default()
        },
    );

    let burn_operation = SetBurnMechanismOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(TRUSTED_TOKEN),
        nonce: state.common_setup.next_operation_nonce(),
    };
    let burn_operation_hash = burn_operation.generate_hash();
    let burn_hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&burn_operation_hash.to_vec()));

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            OWNER_ADDRESS.to_managed_address(),
            None,
        ),
    );
    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    // Deploy and register validators
    state.common_setup.deploy_chain_config(
        OptionalValue::Some(SovereignConfig::default_config_for_test()),
        None,
    );

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);
    let (signature_burn, public_keys_burn) = state
        .common_setup
        .get_sig_and_pub_keys(1, &burn_hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );
    state.common_setup.register(
        public_keys_burn.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );
    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);
    state.common_setup.deploy_testing_sc();

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        state.common_setup.full_bitmap(1),
        0,
        MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()])),
    );

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature_burn,
        &burn_hash_of_hashes,
        state.common_setup.bitmap_for_signers(&[1]),
        0,
        MultiValueEncoded::from(ManagedVec::from(vec![burn_operation_hash])),
    );

    state.set_token_burn_mechanism(&burn_hash_of_hashes, burn_operation);
    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        None,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
    state.common_setup.check_account_single_esdt(
        TESTING_SC_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN),
        0u64,
        BigUint::zero(),
    );
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with transfer data only
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[test]
fn test_execute_operation_only_transfer_data_no_fee() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        ManagedVec::new(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            OWNER_ADDRESS.to_managed_address(),
            Some(transfer_data),
        ),
    );

    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    state.common_setup.deploy_chain_config(
        OptionalValue::Some(SovereignConfig::default_config_for_test()),
        None,
    );

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.deploy_testing_sc();

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        operations_hashes,
    );

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        None,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with burn mechanism after deposit
///
/// ### EXPECTED
/// The operation executes successfully, tokens are burned
#[test]
fn test_execute_operation_success_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    let amount = BigUint::from(ONE_HUNDRED_THOUSAND);
    let payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(TRUSTED_TOKEN),
        0,
        EsdtTokenData {
            amount: amount.clone(),
            ..Default::default()
        },
    );

    let burn_operation = SetBurnMechanismOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(TRUSTED_TOKEN),
        nonce: state.common_setup.next_operation_nonce(),
    };
    let burn_operation_hash = burn_operation.generate_hash();
    let burn_hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&burn_operation_hash.to_vec()));

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment.clone()].into(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            OWNER_ADDRESS.to_managed_address(),
            None,
        ),
    );
    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state.common_setup.deploy_chain_config(
        OptionalValue::Some(SovereignConfig::default_config_for_test()),
        None,
    );

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);
    let (signature_burn, public_keys_burn) = state
        .common_setup
        .get_sig_and_pub_keys(1, &burn_hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );
    state.common_setup.register(
        public_keys_burn.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );
    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);
    state.common_setup.deploy_testing_sc();

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::from(vec![payment]),
        None,
        Some(DEPOSIT_EVENT),
    );

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        state.common_setup.full_bitmap(1),
        0,
        MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()])),
    );

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature_burn,
        &burn_hash_of_hashes,
        state.common_setup.bitmap_for_signers(&[1]),
        0,
        MultiValueEncoded::from(ManagedVec::from(vec![burn_operation_hash])),
    );

    state.set_token_burn_mechanism(&burn_hash_of_hashes, burn_operation);
    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        None,
    );

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(TRUSTED_TOKEN),
            &(BigUint::from(ONE_HUNDRED_MILLION) - &amount),
        );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN),
        0u64,
        BigUint::zero(),
    );

    state
        .common_setup
        .check_deposited_tokens_amount(vec![(EgldOrEsdtTokenIdentifier::esdt(TRUSTED_TOKEN), 0)]);

    state.common_setup.check_account_single_esdt(
        TESTING_SC_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN),
        0u64,
        amount,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' after switching between LOCK and BURN mechanisms
///
/// ### EXPECTED
/// - Operations execute successfully in different mechanism states
/// - Token balances are tracked correctly during mechanism switches
#[test]
fn test_deposit_execute_switch_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();

    // === Initial Setup ===
    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    let chain_config_config = SovereignConfig {
        max_validators: 4,
        ..SovereignConfig::default_config_for_test()
    };
    state
        .common_setup
        .deploy_chain_config(OptionalValue::Some(chain_config_config), None);

    let trusted_token_id = TRUSTED_TOKEN;
    let execute_amount = 500u64;
    let deposit_amount = 1000u64;

    // === Setup Operations ===
    let execute_payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(trusted_token_id),
        0,
        EsdtTokenData {
            amount: BigUint::from(execute_amount),
            ..Default::default()
        },
    );

    let burn_operation = SetBurnMechanismOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(TRUSTED_TOKEN),
        nonce: state.common_setup.next_operation_nonce(),
    };
    let burn_operation_hash = burn_operation.generate_hash();
    let burn_operation_hash_of_hashes =
        ManagedBuffer::new_from_bytes(&sha256(&burn_operation_hash.to_vec()));
    let (signature_burn, pub_keys_burn) = state
        .common_setup
        .get_sig_and_pub_keys(1, &burn_operation_hash_of_hashes);
    state.common_setup.register(
        pub_keys_burn.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    let operation_one = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![execute_payment.clone()].into(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            OWNER_ADDRESS.to_managed_address(),
            None,
        ),
    );
    let operation_one_hash = state.common_setup.get_operation_hash(&operation_one);
    let hash_of_hashes_one = ManagedBuffer::new_from_bytes(&sha256(&operation_one_hash.to_vec()));
    let (signature_one, pub_keys_one) = state
        .common_setup
        .get_sig_and_pub_keys(1, &hash_of_hashes_one);
    state.common_setup.register(
        pub_keys_one.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    let lock_operation = SetLockMechanismOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(TRUSTED_TOKEN),
        nonce: state.common_setup.next_operation_nonce(),
    };
    let lock_operation_hash = lock_operation.generate_hash();
    let lock_operation_hash_of_hashes =
        ManagedBuffer::new_from_bytes(&sha256(&lock_operation_hash.to_vec()));
    let (signature_lock, pub_keys_lock) = state
        .common_setup
        .get_sig_and_pub_keys(1, &lock_operation_hash_of_hashes);
    state.common_setup.register(
        pub_keys_lock.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    let operation_two = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![execute_payment].into(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            OWNER_ADDRESS.to_managed_address(),
            None,
        ),
    );
    let operation_two_hash = state.common_setup.get_operation_hash(&operation_two);
    let hash_of_hashes_two = ManagedBuffer::new_from_bytes(&sha256(&operation_two_hash.to_vec()));
    let (signature_two, pub_keys_two) = state
        .common_setup
        .get_sig_and_pub_keys(1, &hash_of_hashes_two);
    state.common_setup.register(
        pub_keys_two.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    // === Complete Remaining Setup ===
    state.common_setup.complete_chain_config_setup_phase();
    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);
    state.common_setup.deploy_testing_sc();

    let deposit_payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(trusted_token_id),
        0,
        EsdtTokenData {
            amount: BigUint::from(deposit_amount),
            ..Default::default()
        },
    );

    // === Test Flow ===
    // 1. First deposit (default LOCK mechanism)
    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::from(vec![deposit_payment.clone()]),
        None,
        Some(DEPOSIT_EVENT),
    );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0,
        BigUint::from(deposit_amount),
    );

    // 2. Switch to BURN mechanism (uses validator 0)
    let burn_bitmap = state.common_setup.bitmap_for_signers(&[0]);
    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature_burn,
        &burn_operation_hash_of_hashes,
        burn_bitmap,
        0,
        MultiValueEncoded::from(ManagedVec::from(vec![burn_operation_hash.clone()])),
    );
    state.set_token_burn_mechanism(&burn_operation_hash_of_hashes, burn_operation);

    let mut expected_deposited = deposit_amount;
    state.common_setup.check_deposited_tokens_amount(vec![(
        EgldOrEsdtTokenIdentifier::esdt(trusted_token_id),
        expected_deposited,
    )]);
    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0,
        BigUint::zero(),
    );

    // 3. Execute operation 1 (BURN mechanism, uses validator 1)
    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature_one,
        &hash_of_hashes_one,
        state.common_setup.bitmap_for_signers(&[1]),
        0,
        MultiValueEncoded::from(ManagedVec::from(vec![operation_one_hash])),
    );

    state.execute_operation(
        &hash_of_hashes_one,
        &operation_one,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        None,
    );

    let mut expected_receiver = execute_amount;
    expected_deposited -= execute_amount;

    state.common_setup.check_deposited_tokens_amount(vec![(
        EgldOrEsdtTokenIdentifier::esdt(trusted_token_id),
        expected_deposited,
    )]);
    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0,
        BigUint::zero(),
    );

    // 4. Second deposit (BURN mechanism)
    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::from(vec![deposit_payment.clone()]),
        None,
        Some(DEPOSIT_EVENT),
    );

    expected_deposited += deposit_amount;
    state.common_setup.check_deposited_tokens_amount(vec![(
        EgldOrEsdtTokenIdentifier::esdt(trusted_token_id),
        expected_deposited,
    )]);
    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0,
        BigUint::zero(),
    );

    // 5. Switch back to LOCK mechanism (uses validator 2)
    let lock_bitmap = state.common_setup.bitmap_for_signers(&[2]);
    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature_lock,
        &lock_operation_hash_of_hashes,
        lock_bitmap,
        0,
        MultiValueEncoded::from(ManagedVec::from(vec![lock_operation_hash.clone()])),
    );
    state.set_token_lock_mechanism(&lock_operation_hash_of_hashes, lock_operation);

    state.common_setup.check_deposited_tokens_amount(vec![(
        EgldOrEsdtTokenIdentifier::esdt(trusted_token_id),
        0,
    )]);
    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0,
        BigUint::from(expected_deposited),
    );

    // 6. Execute operation 2 (LOCK mechanism, uses validator 3)
    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature_two,
        &hash_of_hashes_two,
        state.common_setup.bitmap_for_signers(&[3]),
        0,
        MultiValueEncoded::from(ManagedVec::from(vec![operation_two_hash])),
    );

    state.execute_operation(
        &hash_of_hashes_two,
        &operation_two,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        None,
    );

    expected_receiver += execute_amount;
    expected_deposited -= execute_amount;

    state.common_setup.check_deposited_tokens_amount(vec![(
        EgldOrEsdtTokenIdentifier::esdt(trusted_token_id),
        0,
    )]);
    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0,
        BigUint::from(expected_deposited),
    );
    state.common_setup.check_account_single_esdt(
        TESTING_SC_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0,
        BigUint::from(expected_receiver),
    );

    // 7. Third deposit (LOCK mechanism)
    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::from(vec![deposit_payment]),
        None,
        Some(DEPOSIT_EVENT),
    );

    expected_deposited += deposit_amount;

    state.common_setup.check_deposited_tokens_amount(vec![(
        EgldOrEsdtTokenIdentifier::esdt(trusted_token_id),
        0,
    )]);
    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0,
        BigUint::from(expected_deposited),
    );
    state.common_setup.check_account_single_esdt(
        TESTING_SC_ADDRESS.to_address(),
        TestTokenIdentifier::new(trusted_token_id),
        0,
        BigUint::from(expected_receiver),
    );
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with empty payments
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[test]
fn test_execute_operation_no_payments() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        ManagedVec::new(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            OWNER_ADDRESS.to_managed_address(),
            Some(transfer_data),
        ),
    );

    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state.common_setup.deploy_chain_config(
        OptionalValue::Some(SovereignConfig::default_config_for_test()),
        None,
    );

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.deploy_testing_sc();

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        operations_hashes,
    );

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        None,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with empty payments and wrong endpoint
///
/// ### EXPECTED
/// The operation is not executed in the testing smart contract
#[test]
fn test_execute_operation_no_payments_failed_event() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state.common_setup.deploy_chain_config(
        OptionalValue::Some(SovereignConfig::default_config_for_test()),
        None,
    );

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from(WRONG_ENDPOINT_NAME);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);
    let transfer_data = TransferData::new(gas_limit, function, args);
    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        ManagedVec::new(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            OWNER_ADDRESS.to_managed_address(),
            Some(transfer_data),
        ),
    );
    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.deploy_testing_sc();

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        operations_hashes,
    );

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        Some(INVALID_FUNCTION_NOT_FOUND),
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}

/// ### TEST
/// M-NATIVE_ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with native esdt payment and wrong endpoint
///
/// ### EXPECTED
/// The operation is not executed in the testing smart contract
/// Native ESDT should be burned
#[test]
fn test_execute_operation_native_token_failed_event() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state.common_setup.deploy_chain_config(
        OptionalValue::Some(SovereignConfig::default_config_for_test()),
        None,
    );

    let token_data = EsdtTokenData {
        amount: BigUint::from(ONE_HUNDRED_TOKENS),
        ..Default::default()
    };
    let payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(NATIVE_TEST_TOKEN),
        0,
        token_data,
    );

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from(WRONG_ENDPOINT_NAME);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);
    let transfer_data = TransferData::new(gas_limit, function, args);
    let operation_data = OperationData::new(
        state.common_setup.next_operation_nonce(),
        OWNER_ADDRESS.to_managed_address(),
        Some(transfer_data),
    );
    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );
    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.deploy_testing_sc();

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));
    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        operations_hashes,
    );

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(vec![EXECUTED_BRIDGE_OP_EVENT]),
        Some(INVALID_FUNCTION_NOT_FOUND),
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);

    state.common_setup.check_account_single_esdt(
        OWNER_ADDRESS.to_address(),
        NATIVE_TEST_TOKEN,
        0u64,
        BigUint::zero(),
    );

    state.common_setup.check_account_single_esdt(
        TESTING_SC_ADDRESS.to_address(),
        NATIVE_TEST_TOKEN,
        0u64,
        BigUint::zero(),
    );
}

/// ### TEST
/// M-ESDT_SET_BURN_FAIL
///
/// ### ACTION
/// Call 'set_token_burn_mechanism()' without the proper roles
///
/// ### EXPECTED
/// Error MINT_AND_BURN_ROLES_NOT_FOUND
#[test]
fn test_set_token_burn_mechanism_no_roles() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state.set_token_burn_mechanism_before_setup_phase("WEGLD", Some(MINT_AND_BURN_ROLES_NOT_FOUND));
}

/// ### TEST
/// M-ESDT_SET_BURN_FAIL
///
/// ### ACTION
/// Call 'set_token_burn_mechanism()' without a trusted token id
///
/// ### EXPECTED
/// Error TOKEN_ID_IS_NOT_TRUSTED
#[test]
fn test_set_token_burn_mechanism_token_not_trusted() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);
    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state.set_token_burn_mechanism_before_setup_phase(
        FIRST_TEST_TOKEN.as_str(),
        Some(TOKEN_ID_IS_NOT_TRUSTED),
    );
}

/// ### TEST
/// M-ESDT_SET_BURN_OK
///
/// ### ACTION
/// Call 'set_token_burn_mechanism()' with a trusted token id
///
/// ### EXPECTED
/// The trusted token has the burn mechanism set
#[test]
fn test_set_token_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state.set_token_burn_mechanism_before_setup_phase(TRUSTED_TOKEN, None);

    state
        .common_setup
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc
                .burn_mechanism_tokens()
                .contains(&EgldOrEsdtTokenIdentifier::esdt(TRUSTED_TOKEN)))
        });

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN),
        0u64,
        BigUint::zero(),
    );
}

/// ### TEST
/// M-ESDT_SET_BURN_OK
///
/// ### ACTION
/// Call both 'set_token_burn_mechanism()' and 'set_token_lock_mechanism()' with a trusted token id.
///
/// ### EXPECTED
/// The trusted token has the lock mechanism set
#[test]
fn test_set_token_lock_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);

    state.set_token_burn_mechanism_before_setup_phase(TRUSTED_TOKEN, None);
    state.set_token_lock_mechanism_before_setup_phase(TRUSTED_TOKEN, None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

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
        TestTokenIdentifier::new(TRUSTED_TOKEN),
        100u64,
        BigUint::zero(),
    );
}

/// ### TEST
/// M-ESDT_UPDATE_CONFIG_FAIL
///
/// ### ACTION
/// Call `update_esdt_safe_config()` before setup phase completion
///
/// ### EXPECTED
/// ERROR SETUP_PHASE_NOT_COMPLETED
#[test]
fn test_update_config_setup_phase_not_completed() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);

    let esdt_safe_config = EsdtSafeConfig::default_config();

    let nonce = state.common_setup.next_operation_nonce();
    state.update_esdt_safe_config(
        &ManagedBuffer::new(),
        UpdateEsdtSafeConfigOperation {
            esdt_safe_config,
            nonce,
        },
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(SETUP_PHASE_NOT_COMPLETED),
    );
}

/// ### TEST
/// M-ESDT_UPDATE_CONFIG_FAIL
///
/// ### ACTION
/// Call `update_esdt_safe_config()` before registering operation
///
/// ### EXPECTED
/// ERROR CURRENT_OPERATION_NOT_REGISTERED
#[test]
fn test_update_config_operation_not_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    let esdt_safe_config = EsdtSafeConfig::default_config();

    let nonce = state.common_setup.next_operation_nonce();
    state.update_esdt_safe_config(
        &ManagedBuffer::new(),
        UpdateEsdtSafeConfigOperation {
            esdt_safe_config,
            nonce,
        },
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(CURRENT_OPERATION_NOT_REGISTERED),
    );
}

/// ### TEST
/// M-ESDT_UPDATE_CONFIG_ERROR
///
/// ### ACTION
/// Call `update_esdt_safe_config()` with an invalid config
///
/// ### EXPECTED
/// failedBridgeOp event is emitted
#[test]
fn test_update_config_invalid_config() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state.common_setup.deploy_chain_config(
        OptionalValue::Some(SovereignConfig::default_config_for_test()),
        None,
    );

    let esdt_safe_config = EsdtSafeConfig {
        max_tx_gas_limit: MAX_GAS_PER_TRANSACTION + 1,
        ..EsdtSafeConfig::default_config()
    };
    let update_config_operation = UpdateEsdtSafeConfigOperation {
        esdt_safe_config: esdt_safe_config.clone(),
        nonce: state.common_setup.next_operation_nonce(),
    };

    let config_hash = update_config_operation.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&config_hash.to_vec()));

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);
    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![config_hash]),
    );

    state.update_esdt_safe_config(
        &hash_of_hashes,
        update_config_operation,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(MAX_GAS_LIMIT_PER_TX_EXCEEDED),
    );
}

/// ### TEST
/// M-ESDT_UPDATE_CONFIG_OK
///
/// ### ACTION
/// Call `update_esdt_safe_config()`
///
/// ### EXPECTED
/// EsdtSafeConfig is updated and executedBridgeOp is emitted
#[test]
fn test_update_config() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state.common_setup.deploy_chain_config(
        OptionalValue::Some(SovereignConfig::default_config_for_test()),
        None,
    );

    let esdt_safe_config = EsdtSafeConfig {
        max_tx_gas_limit: 100_000,
        ..EsdtSafeConfig::default_config()
    };
    let update_config_operation = UpdateEsdtSafeConfigOperation {
        esdt_safe_config,
        nonce: state.common_setup.next_operation_nonce(),
    };

    let config_hash = update_config_operation.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&config_hash.to_vec()));

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![config_hash]),
    );

    state.update_esdt_safe_config(
        &hash_of_hashes,
        update_config_operation,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        None,
    );

    state
        .common_setup
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            let config = sc.esdt_safe_config().get();
            assert!(config.max_tx_gas_limit == 100_000);
        });

    state
        .common_setup
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let new_config_whitebox = EsdtSafeConfig {
                max_tx_gas_limit: 100_000,
                ..EsdtSafeConfig::default_config()
            };

            let config_hash_whitebox = new_config_whitebox.generate_hash();
            let hash_of_hashes_whitebox =
                ManagedBuffer::new_from_bytes(&sha256(&config_hash_whitebox.to_vec()));
            assert!(sc
                .operation_hash_status(&hash_of_hashes_whitebox, &config_hash_whitebox)
                .is_empty())
        });
}

#[test]
fn test_execute_operation_partial_execution() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles(None);
    state.set_token_burn_mechanism_before_setup_phase(TRUSTED_TOKEN, None);
    state.complete_setup_phase(Some(UNPAUSE_CONTRACT_LOG));

    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            sc.multiversx_to_sovereign_token_id_mapper(&EgldOrEsdtTokenIdentifier::esdt(
                FIRST_TOKEN_ID,
            ))
            .set(EgldOrEsdtTokenIdentifier::esdt(
                SOV_FIRST_TOKEN_ID.to_token_identifier(),
            ));
            sc.multiversx_to_sovereign_token_id_mapper(&EgldOrEsdtTokenIdentifier::esdt(
                SECOND_TOKEN_ID,
            ))
            .set(EgldOrEsdtTokenIdentifier::esdt(
                SOV_SECOND_TOKEN_ID.to_token_identifier(),
            ));
            sc.sovereign_to_multiversx_token_id_mapper(&EgldOrEsdtTokenIdentifier::esdt(
                SOV_FIRST_TOKEN_ID.to_token_identifier(),
            ))
            .set(EgldOrEsdtTokenIdentifier::esdt(
                FIRST_TOKEN_ID.to_token_identifier(),
            ));
            sc.sovereign_to_multiversx_token_id_mapper(&EgldOrEsdtTokenIdentifier::esdt(
                SOV_SECOND_TOKEN_ID.to_token_identifier(),
            ))
            .set(EgldOrEsdtTokenIdentifier::esdt(
                SECOND_TOKEN_ID.to_token_identifier(),
            ));
        });

    state.common_setup.deploy_chain_config(
        OptionalValue::Some(SovereignConfig::default_config_for_test()),
        None,
    );

    let token_data = EsdtTokenData {
        amount: BigUint::from(ONE_HUNDRED_THOUSAND),
        ..Default::default()
    };

    let first_payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(SOV_FIRST_TOKEN_ID),
        0,
        token_data.clone(),
    );

    let second_payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(TRUSTED_TOKEN),
        0,
        token_data.clone(),
    );

    let third_payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(SOV_SECOND_TOKEN_ID),
        0,
        token_data,
    );

    let operation = Operation::new(
        USER_ADDRESS.to_managed_address(),
        vec![first_payment, second_payment, third_payment].into(),
        OperationData::new(
            state.common_setup.next_operation_nonce(),
            USER_ADDRESS.to_managed_address(),
            None,
        ),
    );
    let operation_hash = state.common_setup.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::ESDTSafe]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = state.common_setup.full_bitmap(1);
    let epoch = 0;

    state.common_setup.register_operation(
        USER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![operation_hash]),
    );

    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(vec![
            EXECUTED_BRIDGE_OP_EVENT,
            DEPOSIT_EVENT,
            &SOV_FIRST_TOKEN_ID.as_str(),
            &TRUSTED_TOKEN,
            &SOV_SECOND_TOKEN_ID.as_str(),
        ]),
        None,
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        FIRST_TOKEN_ID,
        0,
        BigUint::zero(),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        SECOND_TOKEN_ID,
        0,
        BigUint::zero(),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN),
        0,
        BigUint::zero(),
    );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TOKEN_ID,
        0,
        BigUint::zero(),
    );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        SECOND_TOKEN_ID,
        0,
        BigUint::zero(),
    );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        TestTokenIdentifier::new(TRUSTED_TOKEN),
        0,
        BigUint::zero(),
    );
}
