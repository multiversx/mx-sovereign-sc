use common_blackbox_setup::{
    ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_TOKEN, HEADER_VERIFIER_ADDRESS, ONE_HUNDRED_MILLION,
    ONE_HUNDRED_THOUSAND, OWNER_ADDRESS, TESTING_SC_ADDRESS, TEST_TOKEN_ONE,
    TEST_TOKEN_ONE_WITH_PREFIX, TEST_TOKEN_TWO, USER,
};
use cross_chain::{DEFAULT_ISSUE_COST, MAX_GAS_PER_TRANSACTION};
use error_messages::{
    BANNED_ENDPOINT_NAME, CANNOT_REGISTER_TOKEN, GAS_LIMIT_TOO_HIGH, INVALID_TYPE,
    MAX_GAS_LIMIT_PER_TX_EXCEEDED, NO_ESDT_SAFE_ADDRESS, PAYMENT_DOES_NOT_COVER_FEE,
    TOO_MANY_TOKENS,
};
use header_verifier::OperationHashStatus;
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EsdtTokenData, EsdtTokenPayment, EsdtTokenType, ManagedBuffer, ManagedVec,
        MultiValueEncoded, TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;
use multiversx_sc_scenario::{api::StaticApi, multiversx_chain_vm::crypto_functions::sha256};
use mvx_esdt_safe_blackbox_setup::{MvxEsdtSafeTestState, RegisterTokenArgs};
use proxies::fee_market_proxy::{FeeStruct, FeeType};
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    operation::{Operation, OperationData, OperationEsdtPayment, TransferData},
};

mod mvx_esdt_safe_blackbox_setup;

#[test]
fn deploy() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
}

/// Test that deploy fails when the gas limit in the config is too high
#[test]
fn deploy_invalid_config() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        MAX_GAS_PER_TRANSACTION + 1,
        ManagedVec::new(),
    );

    state.update_configuration(config, Some(MAX_GAS_LIMIT_PER_TX_EXCEEDED));
}

/// Test that deposit fails when there is no payment for transfer
#[test]
fn deposit_nothing_to_transfer() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        None,
        Some("Nothing to transfer"),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(TEST_TOKEN_ONE);
}

/// Test that deposit fails when there are too many tokens in the payment (limit being the MAX_TRANSFERS_PER_TX)
#[test]
fn deposit_too_many_tokens() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    let esdt_token_payment = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::default(),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment; 11]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        Some(payments_vec),
        Some(TOO_MANY_TOKENS),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(TEST_TOKEN_ONE);
}

/// Test that deposit fails when there is no transfer data
#[test]
fn deposit_no_transfer_data() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.common_setup.deploy_fee_market(None);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        Some(payments_vec),
        None,
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(TEST_TOKEN_ONE);
}

/// Test that deposit fails when the gas limit is too high
#[test]
fn deposit_gas_limit_too_high() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(ManagedVec::new(), ManagedVec::new(), 1, ManagedVec::new());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.common_setup.deploy_fee_market(None);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec),
        Some(GAS_LIMIT_TOO_HIGH),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(TEST_TOKEN_ONE);
}

/// Test that deposit fails when the endpoint is banned
#[test]
fn deposit_endpoint_banned() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from("hello")]),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.common_setup.deploy_fee_market(None);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec),
        Some(BANNED_ENDPOINT_NAME),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(TEST_TOKEN_ONE);
}

/// Test that deposit succeeds when the fee is enabled
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Deploy the Fee-Market smart contract
/// 3. Deploy the Testing smart contract
/// 4. Set the Fee-Market address
/// 5. Create the fee payment
/// 6. Create the ESDT token payments
/// 7. Create the payments vector
/// 8. Create the transfer data
/// 9. Call the deposit function
/// 10. Check the balances of the accounts
#[test]
fn deposit_fee_enabled() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

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

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
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
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec.clone()),
        None,
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(TEST_TOKEN_ONE),
            expected_amount_token_one,
        );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(TEST_TOKEN_TWO),
            expected_amount_token_two,
        );

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(TokenIdentifier::from(FEE_TOKEN), expected_amount_token_fee);
}

/// Test that deposit fails when the payment does not cover the fee
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Deploy the Fee-Market smart contract
/// 3. Deploy the Testing smart contract
/// 4. Set the Fee-Market address
/// 5. Create the fee payment
/// 6. Create the ESDT token payments
/// 7. Create the payments vector
/// 8. Create the transfer data
/// 9. Call the deposit function
/// 10. Check the balances of the accounts
#[test]
fn deposit_payment_doesnt_cover_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(TEST_TOKEN_ONE),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(TEST_TOKEN_ONE),
            per_transfer: BigUint::from(1u64),
            per_gas: BigUint::from(1u64),
        },
    };

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 10_000;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec),
        Some(PAYMENT_DOES_NOT_COVER_FEE),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(TEST_TOKEN_ONE);
    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(TEST_TOKEN_TWO);
}

/// Test that after deposit fails the tokens are refunded
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Deploy the Fee-Market smart contract
/// 3. Deploy the Testing smart contract
/// 4. Set the Fee-Market address
/// 5. Create the fee payment
/// 6. Create the ESDT token payments
/// 7. Create the payments vector
/// 8. Create the transfer data
/// 9. Call the deposit function
/// 10. Check the logs
/// 11. Check the balances of the accounts
#[test]
fn deposit_refund() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

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

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
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
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    let logs = state.deposit_with_logs(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec.clone(),
    );

    for log in logs {
        assert!(!log.data.is_empty());
    }

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(TEST_TOKEN_ONE),
            &expected_amount_token_one,
        );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(TEST_TOKEN_TWO),
            &expected_amount_token_two,
        );

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(TokenIdentifier::from(FEE_TOKEN), expected_amount_token_fee);
}

/// Test that register token fails when the token has invalid prefix
#[test]
fn register_token_invalid_type_with_prefix() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE_WITH_PREFIX);
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = TEST_TOKEN_ONE;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, Some(INVALID_TYPE));
    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(TEST_TOKEN_ONE);
}

/// Test that register token works with a valid prefix
#[test]
fn register_token_fungible_token_with_prefix() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE_WITH_PREFIX);
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = TEST_TOKEN_ONE;
    let num_decimals = 3;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, None);

    // TODO: Add check for storage after callback issue is fixed
}

/// Test that register token fails with no prefix
#[test]
fn register_token_fungible_token_no_prefix() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = TEST_TOKEN_ONE;
    let num_decimals = 3;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id,
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

    // TODO: Add check for storage after callback issue is fixed
}

/// Test that register token works with a non-fungible token
#[test]
fn register_token_nonfungible_token() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE_WITH_PREFIX);
    let token_type = EsdtTokenType::NonFungible;
    let token_display_name = "TokenOne";
    let num_decimals = 0;
    let token_ticker = TEST_TOKEN_ONE;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, None);

    // TODO: Add check for storage after callback issue is fixed
}

/// Test that register token fails if the token is already registered
#[test]
fn register_native_token_already_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(
        TEST_TOKEN_ONE,
        token_display_name,
        egld_payment.clone(),
        None,
    );

    // TODO: Add check for storage after callback issue is fixed

    state.register_native_token(
        TEST_TOKEN_ONE,
        token_display_name,
        egld_payment.clone(),
        None,
        // NOTE: Some(NATIVE_TOKEN_ALREADY_REGISTERED) when fix is here,
    );
}

/// Test that register native works in the happy flow
#[test]
fn register_native_token() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(
        TEST_TOKEN_ONE,
        token_display_name,
        egld_payment.clone(),
        None,
    );

    // TODO: Check storage
}

/// Test that execute operation fails when the Mvx-ESDT-Safe address is not set
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Create the operation
/// 3. Create the hash of hashes
/// 4. Deploy the Header-Verifier smart contract
/// 5. Call the execute operation function
/// 6. Check the operation hash status
#[test]
fn execute_operation_no_esdt_safe_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        EsdtTokenData::default(),
    );

    let operation_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let hash_of_hashes = state.get_operation_hash(&operation);

    state.common_setup.deploy_header_verifier();

    state.execute_operation(
        hash_of_hashes.clone(),
        operation,
        Some(NO_ESDT_SAFE_ADDRESS),
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&hash_of_hashes);
}

/// Test that execute operation works in the happy flow
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Create the operation
/// 3. Create the hash of hashes
/// 4. Deploy the Header-Verifier smart contract
/// 5. Deploy the Testing smart contract
/// 6. Set the Mvx-ESDT-Safe address in the Header-Verifier smart contract
/// 7. Call the register operation function
/// 8. Call the execute operation function
/// 9. Check the operation hash status
#[test]
fn execute_operation_success() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(TokenIdentifier::from(TEST_TOKEN_ONE), 0, token_data);

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

    let operation_hash = state.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state.common_setup.deploy_header_verifier();
    state.common_setup.deploy_testing_sc();
    state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(hash_of_hashes, operation.clone(), None);

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}

/// Test execute operation with native token happy flow
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Register the native token
/// 3. Create the operation
/// 4. Create the hash of hashes
/// 5. Deploy the Header-Verifier smart contract
/// 6. Deploy the Testing smart contract
/// 7. Set the Mvx-ESDT-Safe address in the Header-Verifier smart contract
/// 8. Call the register operation function
/// 9. Call the execute operation function
/// 10. Check the operation hash status
#[test]
fn execute_operation_with_native_token_success() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(TEST_TOKEN_ONE, token_display_name, egld_payment, None);

    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(TokenIdentifier::from(TEST_TOKEN_ONE), 0, token_data);

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

    let operation_hash = state.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state.common_setup.deploy_header_verifier();
    state.common_setup.deploy_testing_sc();
    state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(hash_of_hashes, operation.clone(), None);

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}
