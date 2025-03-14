use esdt_safe_blackbox_setup::*;
use multiversx_sc::{
    imports::OptionalValue,
    types::{ManagedBuffer, TokenIdentifier},
};

mod esdt_safe_blackbox_setup;

#[test]
fn test_deploy() {
    let mut state = EsdtSafeTestState::new();

    state.deploy_esdt_safe_contract(false, OptionalValue::None);
}

#[test]
fn test_main_to_sov_egld_deposit_nothing_to_transfer() {
    let mut state = EsdtSafeTestState::new();
    let err_message = "Nothing to transfer";

    state.deploy_esdt_safe_contract(false, OptionalValue::None);

    state.propose_egld_deposit_and_expect_err(Some(err_message));
}

#[test]
fn test_main_to_sov_deposit_ok() {
    let mut state = EsdtSafeTestState::new();

    state.deploy_esdt_safe_contract(false, OptionalValue::None);

    state.propose_esdt_deposit();
}

#[test]
fn test_execute_operation_not_registered() {
    let mut state = EsdtSafeTestState::new();
    let err_message = "The current operation is not registered";

    state.deploy_esdt_safe_contract(false, OptionalValue::None);

    state.deploy_header_verifier_contract();

    state.propose_set_header_verifier_address();

    state.propose_set_esdt_safe_address();

    state.propose_execute_operation_and_expect_err(Some(err_message));
}

#[test]
fn test_register_operation() {
    let mut state = EsdtSafeTestState::new();

    state.deploy_esdt_safe_contract(false, OptionalValue::None);

    state.deploy_header_verifier_contract();

    state.propose_set_header_verifier_address();

    state.propose_register_operation();
}

#[test]
fn test_register_token_no_prefix() {
    let mut state = EsdtSafeTestState::new();

    state.deploy_esdt_safe_contract(
        false,
        OptionalValue::Some(ManagedBuffer::from("USDC-12345")),
    );
    state.register_token("WEGLD-12345", Some("Token Id does not have prefix"));
}

#[test]
fn test_register_token_not_native() {
    let mut state = EsdtSafeTestState::new();

    state.deploy_esdt_safe_contract(
        false,
        OptionalValue::Some(ManagedBuffer::from("USDC-12345")),
    );
    state.register_token(
        "sov-WEGLD-12345",
        Some("The current token is not the native one"),
    );
}

#[test]
fn test_register_token() {
    let mut state = EsdtSafeTestState::new();

    state.deploy_esdt_safe_contract(
        false,
        OptionalValue::Some(ManagedBuffer::from("USDC-12345")),
    );
    state.register_token(
        "sov-USDC-12345",
        Some("The current token is not the native one"),
    );
}

#[test]
fn test_execute_operation() {
    let mut state = EsdtSafeTestState::new();

    state.deploy_esdt_safe_contract(false, OptionalValue::None);

    state.deploy_header_verifier_contract();

    state.propose_set_header_verifier_address();

    state.propose_set_esdt_safe_address();

    state.propose_register_operation();

    state.propose_execute_operation();
}
