use cross_chain::deposit_unit_tests_setup::{
    CONTRACT_ADDRESS, FEE_MARKET_ADDRESS, ONE_HUNDRED_MILLION, ONE_HUNDRED_THOUSAND, OWNER_ADDRESS,
    TEST_TOKEN_ONE, TEST_TOKEN_TWO, TOKEN_HANDLER_ADDRESS, USER,
};
use multiversx_sc::{
    imports::OptionalValue,
    types::{BigUint, EsdtTokenPayment, TestTokenIdentifier},
};
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::{api::StaticApi, ScenarioTxWhitebox};
use operation::aliases::PaymentsVec;
use setup::SovEnshrineEsdtSafeTestState;

mod setup;

#[test]
fn deploy() {
    let mut state = SovEnshrineEsdtSafeTestState::new();

    state.deploy_contract(TOKEN_HANDLER_ADDRESS);
}

#[test]
fn depost_contract_paused() {
    let mut state = SovEnshrineEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state.deploy_fee_market(None);
    state.deploy_testing_sc();
    // state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(TEST_TOKEN_ONE);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        test_token_one_identifier.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one.clone()]);

    state.deposit_with(
        USER.to_managed_address(),
        OptionalValue::None,
        payments_vec.clone(),
        Some("Cannot create transaction while paused"),
        false,
    );
}

#[test]
fn depost() {
    let mut state = SovEnshrineEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state.deploy_fee_market(None);
    state.deploy_testing_sc();
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(CONTRACT_ADDRESS)
        .whitebox(sov_enshrine_esdt_safe::contract_obj, |sc| {
            sc.set_paused(false);
        });
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

    state.deposit_with(
        USER.to_managed_address(),
        OptionalValue::None,
        payments_vec.clone(),
        Some("Cannot create transaction while paused"),
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
