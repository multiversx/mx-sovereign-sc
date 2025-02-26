use cross_chain::deposit_unit_tests_setup::{
    FEE_MARKET_ADDRESS, FEE_TOKEN, ONE_HUNDRED_MILLION, ONE_HUNDRED_THOUSAND, OWNER_ADDRESS,
    TEST_TOKEN_ONE, TEST_TOKEN_TWO, TOKEN_HANDLER_ADDRESS, USER,
};
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{BigUint, EsdtTokenPayment, ManagedBuffer, ManagedVec, TestTokenIdentifier},
};
use multiversx_sc_scenario::api::StaticApi;
use operation::{aliases::PaymentsVec, EsdtSafeConfig, SovereignConfig};
use proxies::fee_market_proxy::{FeeStruct, FeeType};
use sov_enshrine_unit_tests_setup::SovEnshrineEsdtSafeTestState;

mod sov_enshrine_unit_tests_setup;

#[test]
fn deploy() {
    let mut state = SovEnshrineEsdtSafeTestState::new();
    let config = Some(EsdtSafeConfig::default_config());

    state.deploy_contract(TOKEN_HANDLER_ADDRESS, config);
}

#[test]
fn depost_contract_paused() {
    let mut state = SovEnshrineEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state.deploy_fee_market(None);
    state.deploy_testing_sc();

    let test_token_one_identifier = TestTokenIdentifier::new(TEST_TOKEN_ONE);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        test_token_one_identifier.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one.clone()]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        Some(payments_vec.clone()),
        Some("Cannot create transaction while paused"),
        false,
    );
}

#[test]
fn deposit_no_transfer_data_no_fee() {
    let mut state = SovEnshrineEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state.deploy_fee_market(None);
    state.deploy_testing_sc();
    state.deploy_chain_config(SovereignConfig::default_config(), OWNER_ADDRESS);
    state.unpause_contract();
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

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        Some(payments_vec.clone()),
        None,
        false,
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, &expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_two_identifier, &expected_amount_token_two);
}

#[test]
fn deposit_with_fee_no_transfer_data() {
    let mut state = SovEnshrineEsdtSafeTestState::new();

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

    state.deploy_fee_market(Some(fee));
    state.deploy_testing_sc();
    state.deploy_chain_config(SovereignConfig::default_config(), OWNER_ADDRESS);
    state.unpause_contract();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let test_token_two_identifier = TestTokenIdentifier::new(TEST_TOKEN_TWO);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(fee_token_identifier.into(), 0, fee_amount.clone());

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
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        Some(payments_vec.clone()),
        None,
        false,
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, &expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_two_identifier, &expected_amount_token_two);

    let expected_amount_token_fee =
        BigUint::from(ONE_HUNDRED_MILLION) - BigUint::from(payments_vec.len() - 1) * per_transfer;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(fee_token_identifier, expected_amount_token_fee);
}

#[test]
fn deposit_with_fee_with_transfer_data() {
    let mut state = SovEnshrineEsdtSafeTestState::new();

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

    state.deploy_fee_market(Some(fee));
    state.deploy_testing_sc();
    state.unpause_contract();
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
        false,
    );

    // TODO:
    // state
    //     .world
    //     .check_account(CONTRACT_ADDRESS)
    //     .esdt_balance(test_token_one_identifier, BigUint::from(0u64));

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_two_identifier, expected_amount_token_two);

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(fee_token_identifier, expected_amount_token_fee);
}
