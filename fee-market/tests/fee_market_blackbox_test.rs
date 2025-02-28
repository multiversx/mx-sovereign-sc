use fee_market_setup::*;

mod fee_market_setup;

#[test]
fn test_deploy_fee_market() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();
}

#[test]
fn test_add_fee_wrong_params() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();

    state.add_fee(WRONG_TOKEN_ID, "Fixed", Some("Invalid token ID"));

    state.add_fee(TOKEN_ID, "None", Some("Invalid fee type"));

    state.add_fee(DIFFERENT_TOKEN_ID, "Fixed", Some("Invalid fee"));

    state.add_fee(TOKEN_ID, "AnyTokenWrong", Some("Invalid token ID"));
}

#[test]
fn test_substract_fee_no_fee() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();
    state.remove_fee();

    state.substract_fee("Correct", None);

    state.check_balance_sc(ESDT_SAFE_ADDRESS, 1000);
    state.check_account(USER_ADDRESS, 1000);
}

#[test]
fn test_substract_fee_whitelisted() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();
    state.add_users_to_whitelist();

    state.substract_fee("Correct", None);

    state.check_balance_sc(ESDT_SAFE_ADDRESS, 1000);
    state.check_account(USER_ADDRESS, 1000);
}

#[test]
fn test_substract_fee_invalid_payment_token() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();

    state.substract_fee("InvalidToken", Some("Token not accepted as fee"));

    state.check_balance_sc(ESDT_SAFE_ADDRESS, 1000);
    state.check_account(USER_ADDRESS, 1000);
}

#[test]
fn test_substract_fixed_fee_payment_not_covered() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();

    state.substract_fee(
        "Less than fee",
        Some("Payment does not cover fee"),
    );

    state.check_balance_sc(ESDT_SAFE_ADDRESS, 1000);
    state.check_account(USER_ADDRESS, 1000);
}

#[test]
fn test_substract_fee_fixed_payment_bigger_than_fee() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();

    state.substract_fee("Correct", None);

    state.check_balance_sc(ESDT_SAFE_ADDRESS, 800);
    state.check_account(USER_ADDRESS, 1100);
}
