use common_test_setup::constants::{
    ESDT_SAFE_ADDRESS, FIRST_TEST_TOKEN, OWNER_BALANCE, SECOND_TEST_TOKEN, USER_ADDRESS,
    WRONG_TOKEN_ID,
};
use error_messages::{
    INVALID_FEE, INVALID_FEE_TYPE, INVALID_TOKEN_ID, PAYMENT_DOES_NOT_COVER_FEE,
    TOKEN_NOT_ACCEPTED_AS_FEE,
};
use fee_market_blackbox_setup::*;
use multiversx_sc::{imports::MultiValue3, types::BigUint};

mod fee_market_blackbox_setup;

#[test]
fn test_deploy_fee_market() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
}

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

#[test]
fn test_substract_fee_no_fee() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.remove_fee();

    state.substract_fee("Correct", None);

    let tokens = vec![MultiValue3::from((
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    ))];

    state
        .common_setup
        .check_account_balance(ESDT_SAFE_ADDRESS.to_address(), tokens.clone());

    state
        .common_setup
        .check_account_balance(USER_ADDRESS.to_address(), tokens);
}

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

    let tokens = vec![MultiValue3::from((
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    ))];

    state
        .common_setup
        .check_account_balance(ESDT_SAFE_ADDRESS.to_address(), tokens.clone());

    state
        .common_setup
        .check_account_balance(USER_ADDRESS.to_address(), tokens);
}

#[test]
fn test_substract_fee_invalid_payment_token() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.substract_fee("InvalidToken", Some(TOKEN_NOT_ACCEPTED_AS_FEE));

    let tokens = vec![MultiValue3::from((
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    ))];

    state
        .common_setup
        .check_account_balance(ESDT_SAFE_ADDRESS.to_address(), tokens.clone());

    state
        .common_setup
        .check_account_balance(USER_ADDRESS.to_address(), tokens);
}

#[test]
fn test_substract_fixed_fee_payment_not_covered() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.substract_fee("Less than fee", Some(PAYMENT_DOES_NOT_COVER_FEE));

    let tokens = vec![MultiValue3::from((
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    ))];

    state
        .common_setup
        .check_account_balance(ESDT_SAFE_ADDRESS.to_address(), tokens.clone());

    state
        .common_setup
        .check_account_balance(USER_ADDRESS.to_address(), tokens);
}

#[test]
fn test_substract_fee_fixed_payment_bigger_than_fee() {
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.substract_fee("Correct", None);

    let token_esdt_address = vec![MultiValue3::from((
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE - 200),
    ))];

    let token_user = vec![MultiValue3::from((
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE + 100),
    ))];

    state
        .common_setup
        .check_account_balance(ESDT_SAFE_ADDRESS.to_address(), token_esdt_address);

    state
        .common_setup
        .check_account_balance(USER_ADDRESS.to_address(), token_user);
}
