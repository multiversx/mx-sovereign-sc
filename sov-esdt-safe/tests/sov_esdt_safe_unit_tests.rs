use common_blackbox_setup::{
    ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_TOKEN, ONE_HUNDRED_MILLION, ONE_HUNDRED_THOUSAND,
    OWNER_ADDRESS, TEST_TOKEN_ONE, TEST_TOKEN_TWO, USER,
};
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EsdtTokenPayment, ManagedBuffer, ManagedVec, TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_scenario::api::StaticApi;
use proxies::fee_market_proxy::{FeeStruct, FeeType};
use sov_esdt_safe_setup::SovEsdtSafeTestState;
use structs::{aliases::PaymentsVec, configs::EsdtSafeConfig};

mod sov_esdt_safe_setup;

#[test]
fn deploy() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract(
        FEE_MARKET_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
}

#[test]
fn deposit_no_fee_no_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state.common_setup.deploy_fee_market(None);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let test_token_two_identifier = TestTokenIdentifier::new(TEST_TOKEN_TWO);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        test_token_one_identifier.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        test_token_two_identifier.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let logs = state.deposit_with_logs(
        USER.to_managed_address(),
        OptionalValue::None,
        payments_vec.clone(),
    );

    for log in logs {
        assert!(!log.topics.is_empty());
    }

    state.common_setup.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
        sov_esdt_safe::contract_obj,
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, &expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_two_identifier, &expected_amount_token_two);
}

#[test]
fn deposit_with_fee_no_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);
    let fee_token_identifier = TestTokenIdentifier::new(FEE_TOKEN);

    let fee = FeeStruct {
        base_token: fee_token_identifier.into(),
        fee_type: FeeType::Fixed {
            token: fee_token_identifier.into(),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let test_token_two_identifier = TestTokenIdentifier::new(TEST_TOKEN_TWO);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(fee_token_identifier.into(), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        test_token_one_identifier.into(),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        test_token_two_identifier.into(),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        Some(payments_vec.clone()),
        None,
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.common_setup.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
        sov_esdt_safe::contract_obj,
    );

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_two_identifier, expected_amount_token_two);

    let expected_amount_token_fee =
        BigUint::from(ONE_HUNDRED_MILLION) - BigUint::from(payments_vec.len() - 1) * per_transfer;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(fee_token_identifier, expected_amount_token_fee);
}

#[test]
fn deposit_no_fee_with_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state.common_setup.deploy_fee_market(None);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let test_token_two_identifier = TestTokenIdentifier::new(TEST_TOKEN_TWO);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        test_token_one_identifier.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        test_token_two_identifier.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![
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
        assert!(!log.topics.is_empty());
    }

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state.common_setup.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
        sov_esdt_safe::contract_obj,
    );

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, &expected_amount_token_one);

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
}

#[test]
fn deposit_with_fee_with_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);
    let fee_token_identifier = TestTokenIdentifier::new(FEE_TOKEN);

    let fee = FeeStruct {
        base_token: fee_token_identifier.into(),
        fee_type: FeeType::Fixed {
            token: fee_token_identifier.into(),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let test_token_two_identifier = TestTokenIdentifier::new(TEST_TOKEN_TWO);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(fee_token_identifier.into(), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        test_token_one_identifier.into(),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        test_token_two_identifier.into(),
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
        .esdt_balance(test_token_one_identifier, expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.common_setup.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
        sov_esdt_safe::contract_obj,
    );

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
        .esdt_balance(fee_token_identifier, expected_amount_token_fee);
}
