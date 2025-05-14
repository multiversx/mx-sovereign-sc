use common_test_setup::constants::{
    CHAIN_FACTORY_SC_ADDRESS, FUNGIBLE_TOKEN_ID, NFT_TOKEN_ID, TOKEN_HANDLER_SC_ADDRESS,
    USER_ADDRESS,
};
use error_messages::{ACTION_IS_NOT_ALLOWED, ENDPOINT_CAN_ONLY_BE_CALLED_BY_ADMIN};
use multiversx_sc::types::{BigUint, EsdtLocalRole};

mod token_handler_blackbox_setup;
use token_handler_blackbox_setup::TokenHandlerTestState;

#[test]
fn test_deploy() {
    let mut state = TokenHandlerTestState::new();

    state.common_setup.deploy_token_handler();
    state.deploy_factory_sc();
}

/// ### TEST
/// T-HANDLER_WHITELIST_ENSRINE_FAIL_001
///
/// ### ACTION
/// Call 'whitelist_caller()' whitout being an admin
///
/// ### EXPECTED
/// Error Endpoint can only be called by admins
#[test]
fn test_whitelist_enshrine_esdt_caller_not_admin() {
    let mut state = TokenHandlerTestState::new();

    state.common_setup.deploy_token_handler();
    state.deploy_factory_sc();
    state.whitelist_caller(
        TOKEN_HANDLER_SC_ADDRESS,
        Some(ENDPOINT_CAN_ONLY_BE_CALLED_BY_ADMIN),
    );
}

/// ### TEST
/// T-HANDLER_WHITELIST_ENSRINE_OK_001
///
/// ### ACTION
/// Call 'whitelist_caller()'
///
/// ### EXPECTED
/// The caller is whitelisted
#[test]
fn test_whitelist_enshrine() {
    let mut state = TokenHandlerTestState::new();

    state.common_setup.deploy_token_handler();
    state.deploy_factory_sc();
    state.whitelist_caller(CHAIN_FACTORY_SC_ADDRESS, None);
}

// NOTE:
// This test at the moment is expected to fail since there is no way
// to give the correct permissions to the TokenHandler SC

/// ### TEST
/// T-HANDLER_TRANSFER_FAIL_001
///
/// ### ACTION
/// Call 'transfer_tokens()'
///
/// ### EXPECTED
/// Error action is not allowed
#[test]
fn test_transfer_tokens_no_payment() {
    let mut state = TokenHandlerTestState::new();
    let token_ids = [NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID];
    let tokens = state.setup_payments(&token_ids.to_vec());
    let esdt_payment = Option::None;
    let opt_transfer_data = Option::None;

    state.common_setup.deploy_token_handler();
    state.deploy_factory_sc();

    state
        .common_setup
        .world
        .set_esdt_balance(CHAIN_FACTORY_SC_ADDRESS, b"NFT_TOKEN_ID", 100);
    state
        .common_setup
        .world
        .set_esdt_balance(CHAIN_FACTORY_SC_ADDRESS, b"FUNGIBLE_TOKEN_ID", 100);

    state.whitelist_caller(CHAIN_FACTORY_SC_ADDRESS, None);

    state.common_setup.world.set_esdt_local_roles(
        TOKEN_HANDLER_SC_ADDRESS,
        b"NFT_TOKEN_ID",
        &[
            EsdtLocalRole::NftCreate,
            EsdtLocalRole::Mint,
            EsdtLocalRole::NftBurn,
        ],
    );

    state.transfer_tokens(
        CHAIN_FACTORY_SC_ADDRESS,
        esdt_payment,
        opt_transfer_data,
        USER_ADDRESS.to_managed_address(),
        tokens,
        Some(ACTION_IS_NOT_ALLOWED),
    );

    state.common_setup.check_account_single_esdt(
        TOKEN_HANDLER_SC_ADDRESS.to_address(),
        FUNGIBLE_TOKEN_ID,
        0u64,
        BigUint::from(0u64),
    );
}
