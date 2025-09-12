use common_test_setup::constants::{
    ESDT_SAFE_ADDRESS, FIRST_TEST_TOKEN, OWNER_ADDRESS, PER_GAS, PER_TRANSFER, SECOND_TEST_TOKEN,
    SOV_FEE_MARKET_ADDRESS, USER_ADDRESS, WRONG_TOKEN_ID,
};
use error_messages::{
    INVALID_FEE, INVALID_FEE_TYPE, INVALID_PERCENTAGE_SUM, INVALID_TOKEN_ID, ITEM_NOT_IN_LIST,
    PAYMENT_DOES_NOT_COVER_FEE, TOKEN_NOT_ACCEPTED_AS_FEE,
};
use fee_common::storage::FeeCommonStorageModule;
use multiversx_sc::{
    imports::{MultiValue2, OptionalValue},
    types::{BigUint, EgldOrEsdtTokenIdentifier},
};
use multiversx_sc_scenario::ScenarioTxWhitebox;
use sov_fee_market_blackbox_setup::{SovFeeMarketTestState, WantedFeeType};
use structs::fee::{FeeStruct, FeeType};

mod sov_fee_market_blackbox_setup;

/// ### TEST
/// S-FEE-MARKET_DEPLOY_OK
///
/// ### ACTION
/// Deploy sov-fee-market with default config
///
/// ### EXPECTED
/// Contract is deployed with the default config
#[test]
fn test_deploy() {
    let mut state = SovFeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_sov_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
}

/// ### TEST
/// S-FEE-MARKET_SET_FEE_OK
///
/// ### ACTION
/// Call 'set_fee()' with valid fee struct
///
/// ### EXPECTED
/// Fee is set successfully
#[test]
fn test_set_fee() {
    let mut state = SovFeeMarketTestState::new();

    state
        .common_setup
        .deploy_sov_fee_market(None, ESDT_SAFE_ADDRESS);

    let fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
            per_transfer: BigUint::from(PER_TRANSFER),
            per_gas: BigUint::from(PER_GAS),
        },
    };

    state.set_fee(&fee, None);

    state
        .common_setup
        .world
        .query()
        .to(SOV_FEE_MARKET_ADDRESS)
        .whitebox(sov_fee_market::contract_obj, |sc| {
            assert!(!sc
                .token_fee(&EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN))
                .is_empty());
        });
}

/// ### TEST
/// S-FEE-MARKET_SET_FEE_FAIL
///
/// ### ACTION
/// Call 'set_fee()' with invalid token ID
///
/// ### EXPECTED
/// Error INVALID_TOKEN_ID
#[test]
fn test_set_fee_invalid_token() {
    let mut state = SovFeeMarketTestState::new();

    state
        .common_setup
        .deploy_sov_fee_market(None, ESDT_SAFE_ADDRESS);

    let fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(WRONG_TOKEN_ID),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(WRONG_TOKEN_ID),
            per_transfer: BigUint::from(PER_TRANSFER),
            per_gas: BigUint::from(PER_GAS),
        },
    };

    state.set_fee(&fee, Some(INVALID_TOKEN_ID));

    state.check_token_fee_storage_is_empty(WRONG_TOKEN_ID);
}

/// ### TEST
/// S-FEE-MARKET_SET_FEE_FAIL
///
/// ### ACTION
/// Call 'set_fee()' with invalid fee type
///
/// ### EXPECTED
/// Error INVALID_FEE_TYPE
#[test]
fn test_set_fee_invalid_fee_type() {
    let mut state = SovFeeMarketTestState::new();

    state
        .common_setup
        .deploy_sov_fee_market(None, ESDT_SAFE_ADDRESS);

    let fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type: FeeType::None,
    };

    state.set_fee(&fee, Some(INVALID_FEE_TYPE));

    state.check_token_fee_storage_is_empty(FIRST_TEST_TOKEN);
}

/// ### TEST
/// S-FEE-MARKET_SET_FEE_FAIL
///
/// ### ACTION
/// Call 'set_fee()' with invalid fee amount
///
/// ### EXPECTED
/// Error INVALID_FEE
#[test]
fn test_set_fee_invalid_fee() {
    let mut state = SovFeeMarketTestState::new();

    state
        .common_setup
        .deploy_sov_fee_market(None, ESDT_SAFE_ADDRESS);

    let fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(SECOND_TEST_TOKEN),
            per_transfer: BigUint::zero(),
            per_gas: BigUint::zero(),
        },
    };

    state.set_fee(&fee, Some(INVALID_FEE));

    state.check_token_fee_storage_is_empty(FIRST_TEST_TOKEN);
}

/// ### TEST
/// S-FEE-MARKET_REMOVE_FEE_OK
///
/// ### ACTION
/// Call 'remove_fee()' with valid token ID
///
/// ### EXPECTED
/// Fee is removed successfully
#[test]
fn test_remove_fee() {
    let mut state = SovFeeMarketTestState::new();

    let fee = state.get_fee();
    state
        .common_setup
        .deploy_sov_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.remove_fee(FIRST_TEST_TOKEN, None);

    state.check_token_fee_storage_is_empty(FIRST_TEST_TOKEN);
}

/// ### TEST
/// S-FEE-MARKET_REMOVE_FEE_FAIL
///
/// ### ACTION
/// Call 'remove_fee()' with non-existent token ID
///
/// ### EXPECTED
/// No error (should succeed even if token doesn't exist)
#[test]
fn test_remove_fee_non_existent() {
    let mut state = SovFeeMarketTestState::new();

    state
        .common_setup
        .deploy_sov_fee_market(None, ESDT_SAFE_ADDRESS);

    state.remove_fee(SECOND_TEST_TOKEN, None);

    state.check_token_fee_storage_is_empty(SECOND_TEST_TOKEN);
}

/// ### TEST
/// S-FEE-MARKET_SUBTRACT_FEE_OK
///
/// ### ACTION
/// Call 'subtract_fee()' with correct payment
///
/// ### EXPECTED
/// Fee is subtracted successfully
#[test]
fn test_subtract_fee() {
    let mut state = SovFeeMarketTestState::new();

    let fee = state.get_fee();
    state
        .common_setup
        .deploy_sov_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.subtract_fee(
        WantedFeeType::Correct,
        USER_ADDRESS.to_address(),
        1,
        OptionalValue::None,
        None,
    );

    state.check_accumulated_fees(FIRST_TEST_TOKEN, PER_TRANSFER);
}

/// ### TEST
/// S-FEE-MARKET_SUBTRACT_FEE_FAIL
///
/// ### ACTION
/// Call 'subtract_fee()' with invalid token
///
/// ### EXPECTED
/// Error TOKEN_NOT_ACCEPTED_AS_FEE
#[test]
fn test_subtract_fee_invalid_token() {
    let mut state = SovFeeMarketTestState::new();

    let fee = state.get_fee();
    state
        .common_setup
        .deploy_sov_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.subtract_fee(
        WantedFeeType::InvalidToken,
        USER_ADDRESS.to_address(),
        1,
        OptionalValue::None,
        Some(TOKEN_NOT_ACCEPTED_AS_FEE),
    );

    state.check_accumulated_fees(SECOND_TEST_TOKEN, 0);
}

/// ### TEST
/// S-FEE-MARKET_SUBTRACT_FEE_FAIL
///
/// ### ACTION
/// Call 'subtract_fee()' with insufficient payment
///
/// ### EXPECTED
/// Error PAYMENT_DOES_NOT_COVER_FEE
#[test]
fn test_subtract_fee_insufficient_payment() {
    let mut state = SovFeeMarketTestState::new();

    let fee = state.get_fee();
    state
        .common_setup
        .deploy_sov_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.subtract_fee(
        WantedFeeType::LessThanFee,
        USER_ADDRESS.to_address(),
        1,
        OptionalValue::None,
        Some(PAYMENT_DOES_NOT_COVER_FEE),
    );

    state.check_accumulated_fees(FIRST_TEST_TOKEN, 0);
}

/// ### TEST
/// S-FEE-MARKET_DISTRIBUTE_FEES_OK
///
/// ### ACTION
/// Call 'distribute_fees()' with valid address percentage pairs
///
/// ### EXPECTED
/// Fees are distributed successfully
#[test]
fn test_distribute_fees() {
    let mut state = SovFeeMarketTestState::new();

    let fee = state.get_fee();
    state
        .common_setup
        .deploy_sov_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.subtract_fee(
        WantedFeeType::Correct,
        USER_ADDRESS.to_address(),
        1,
        OptionalValue::None,
        None,
    );

    state.check_accumulated_fees(FIRST_TEST_TOKEN, PER_TRANSFER);

    let pairs = vec![
        MultiValue2::from((USER_ADDRESS.to_managed_address(), 5000usize)),
        MultiValue2::from((OWNER_ADDRESS.to_managed_address(), 5000usize)),
    ];

    state.distribute_fees(pairs, None);

    state.check_accumulated_fees(FIRST_TEST_TOKEN, 0);
}

/// ### TEST
/// S-FEE-MARKET_DISTRIBUTE_FEES_FAIL
///
/// ### ACTION
/// Call 'distribute_fees()' with invalid percentage sum
///
/// ### EXPECTED
/// Error about invalid percentage sum
#[test]
fn test_distribute_fees_invalid_percentage() {
    let mut state = SovFeeMarketTestState::new();

    let fee = state.get_fee();
    state
        .common_setup
        .deploy_sov_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    let pairs = vec![
        MultiValue2::from((USER_ADDRESS.to_managed_address(), 6000usize)),
        MultiValue2::from((OWNER_ADDRESS.to_managed_address(), 5000usize)),
    ];

    state.distribute_fees(pairs, Some(INVALID_PERCENTAGE_SUM));

    state.check_accumulated_fees(FIRST_TEST_TOKEN, 0);
}

/// ### TEST
/// S-FEE-MARKET_ADD_USERS_TO_WHITELIST_OK
///
/// ### ACTION
/// Call 'add_users_to_whitelist()' with valid users
///
/// ### EXPECTED
/// Users are added to whitelist successfully
#[test]
fn test_add_users_to_whitelist() {
    let mut state = SovFeeMarketTestState::new();

    state
        .common_setup
        .deploy_sov_fee_market(None, ESDT_SAFE_ADDRESS);

    let users = vec![USER_ADDRESS.to_managed_address()];

    state.add_users_to_whitelist(users, None);

    state
        .common_setup
        .world
        .query()
        .to(SOV_FEE_MARKET_ADDRESS)
        .whitebox(sov_fee_market::contract_obj, |sc| {
            let whitelist = sc.users_whitelist();
            assert!(!whitelist.is_empty());
        });
}

/// ### TEST
/// S-FEE-MARKET_REMOVE_USERS_FROM_WHITELIST_OK
///
/// ### ACTION
/// Call 'remove_users_from_whitelist()' with valid users
///
/// ### EXPECTED
/// Users are removed from whitelist successfully
#[test]
fn test_remove_users_from_whitelist() {
    let mut state = SovFeeMarketTestState::new();

    state
        .common_setup
        .deploy_sov_fee_market(None, ESDT_SAFE_ADDRESS);

    let users = vec![USER_ADDRESS.to_managed_address()];
    state.add_users_to_whitelist(users, None);

    let users_to_remove = vec![USER_ADDRESS.to_managed_address()];
    state.remove_users_from_whitelist(users_to_remove, None);

    state
        .common_setup
        .world
        .query()
        .to(SOV_FEE_MARKET_ADDRESS)
        .whitebox(sov_fee_market::contract_obj, |sc| {
            let whitelist = sc.users_whitelist();
            assert!(whitelist.is_empty());
        });
}

/// ### TEST
/// S-FEE-MARKET_REMOVE_USERS_FROM_WHITELIST_FAIL
///
/// ### ACTION
/// Call 'remove_users_from_whitelist()' with non-whitelisted users
///
/// ### EXPECTED
/// Error ITEM_NOT_IN_LIST
#[test]
fn test_remove_users_from_whitelist_not_in_list() {
    let mut state = SovFeeMarketTestState::new();

    state
        .common_setup
        .deploy_sov_fee_market(None, ESDT_SAFE_ADDRESS);

    let users_to_remove = vec![USER_ADDRESS.to_managed_address()];
    state.remove_users_from_whitelist(users_to_remove, Some(ITEM_NOT_IN_LIST));
}
