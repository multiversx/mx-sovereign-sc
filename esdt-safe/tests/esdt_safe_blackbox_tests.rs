use esdt_safe_blackbox_setup::*;

mod esdt_safe_blackbox_setup;

#[test]
fn test_deploy() {
    let mut state = EsdtSafeTestState::new();

    state.deploy_esdt_safe_contract(false);
}

#[test]
fn test_main_to_sov_egld_deposit_nothing_to_transfer() {
    let mut state = EsdtSafeTestState::new();
    let err_message = "Nothing to transfer";

    state.deploy_esdt_safe_contract(false);

    state.propose_egld_deposit_and_expect_err(Some(err_message));
}

#[test]
fn test_main_to_sov_deposit_ok() {
    let mut state = EsdtSafeTestState::new();

    state.deploy_esdt_safe_contract(false);

    state.propose_esdt_deposit();
}

#[test]
fn test_execute_operation_not_registered() {
    let mut state = EsdtSafeTestState::new();
    let err_message = "The current operation is not registered";

    state.deploy_esdt_safe_contract(false);

    state.deploy_header_verifier_contract();

    state.propose_set_header_verifier_address();

    state.propose_set_esdt_safe_address();

    state.propose_execute_operation_and_expect_err(Some(err_message));
}

#[test]
fn test_register_operation() {
    let mut state = EsdtSafeTestState::new();

    state.deploy_esdt_safe_contract(false);

    state.deploy_header_verifier_contract();

    state.propose_set_header_verifier_address();

    state.propose_register_operation();
}

#[test]
fn test_execute_operation() {
    let mut state = EsdtSafeTestState::new();

    state.deploy_esdt_safe_contract(false);

    state.deploy_header_verifier_contract();

    state.propose_set_header_verifier_address();

    state.propose_set_esdt_safe_address();

    state.propose_register_operation();

    state.propose_execute_operation();
}
