use common_test_setup::constants::{
    ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FIRST_TEST_TOKEN, OWNER_BALANCE, SECOND_TEST_TOKEN,
    USER_ADDRESS, WRONG_TOKEN_ID,
};
use error_messages::{
    INVALID_FEE, INVALID_FEE_TYPE, INVALID_TOKEN_ID, PAYMENT_DOES_NOT_COVER_FEE,
    TOKEN_NOT_ACCEPTED_AS_FEE,
};
use fee_market_blackbox_setup::*;
use multiversx_sc::types::BigUint;

mod fee_market_blackbox_setup;

#[test]
fn test_deploy_fee_market() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
}

/// ### TEST
/// F-MARKET_SET_FEE_FAIL
///
/// ### ACTION
/// Call 'set_fee()' with wrong parameters
///
/// ### EXPECTED
/// Errors: INVALID_TOKEN_ID, INVALID_FEE_TYPE, INVALID_FEE
#[test]
fn test_set_fee_wrong_params() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.set_fee(WRONG_TOKEN_ID, "Fixed", Some(INVALID_TOKEN_ID));

    state.set_fee(FIRST_TEST_TOKEN, "None", Some(INVALID_FEE_TYPE));

    state.set_fee(SECOND_TEST_TOKEN, "Fixed", Some(INVALID_FEE));

    state.set_fee(FIRST_TEST_TOKEN, "AnyTokenWrong", Some(INVALID_TOKEN_ID));
}

/// ### TEST
/// F-MARKET_SUBSTRACT_FEE_OK
///
/// ### ACTION
/// Call 'substract_fee()' with no fee set
///
/// ### EXPECTED
/// User balance is unchanged
#[test]
fn test_substract_fee_no_fee() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.remove_fee();

    state.substract_fee("Correct", None);

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );
}

/// ### TEST
/// F-MARKET_SUBSTRACT_FEE_OK
///
/// ### ACTION
/// Call 'substract_fee()' with a whitelisted user
///
/// ### EXPECTED
/// User balance is unchanged
#[test]
fn test_substract_fee_whitelisted() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    let whitelisted_users = vec![USER_ADDRESS];

    state.add_users_to_whitelist(whitelisted_users);

    state.substract_fee("Correct", None);

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );
}

/// ### TEST
/// F-MARKET_SUBSTRACT_FEE_FAIL
///
/// ### ACTION
/// Call 'substract_fee()' with an invalid payment token
///
/// ### EXPECTED
/// Error TOKEN_NOT_ACCEPTED_AS_FEE
#[test]
fn test_substract_fee_invalid_payment_token() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.substract_fee("InvalidToken", Some(TOKEN_NOT_ACCEPTED_AS_FEE));

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );
}

/// ### TEST
/// F-MARKET_SUBSTRACT_FEE_FAIL
///
/// ### ACTION
/// Call 'substract_fee()' with not enough tokens to cover the fee
///
/// ### EXPECTED
/// Error PAYMENT_DOES_NOT_COVER_FEE
#[test]
fn test_substract_fixed_fee_payment_not_covered() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state
        .common_setup
        .make_header_verifier_owner_of_the_sc(FEE_MARKET_ADDRESS);

    state.substract_fee("Less than fee", Some(PAYMENT_DOES_NOT_COVER_FEE));

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );
}

/// ### TEST
/// F-MARKET_SUBSTRACT_FEE_OK
///
/// ### ACTION
/// Call 'substract_fee()' with payment bigger than fee
///
/// ### EXPECTED
/// User balance is refunded with the difference
#[test]
fn test_substract_fee_fixed_payment_bigger_than_fee() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state
        .common_setup
        .make_header_verifier_owner_of_the_sc(FEE_MARKET_ADDRESS);

    state.substract_fee("Correct", None);

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE - 200),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE + 100),
    );
}
