use chain_config_blackbox_setup::ChainConfigTestState;
use error_messages::INVALID_MIN_MAX_VALIDATOR_NUMBERS;
use multiversx_sc::types::BigUint;
use structs::configs::SovereignConfig;

mod chain_config_blackbox_setup;

#[test]
fn test_deploy_chain_config() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig::new(0, 1, BigUint::default(), None);
    state.common_setup.deploy_chain_config(config);
}

#[test]
fn test_update_config() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig::new(0, 1, BigUint::default(), None);
    state.common_setup.deploy_chain_config(config);

    let new_config = SovereignConfig::new(2, 4, BigUint::default(), None);

    state.update_chain_config(new_config, None);
}

#[test]
fn test_update_config_wrong_validators_array() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig::new(0, 1, BigUint::default(), None);
    state.common_setup.deploy_chain_config(config);

    let new_config = SovereignConfig::new(2, 1, BigUint::default(), None);

    state.update_chain_config(new_config, Some(INVALID_MIN_MAX_VALIDATOR_NUMBERS));
}

#[test]
fn test_complete_setup_phase() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig::new(0, 1, BigUint::default(), None);
    state.common_setup.deploy_chain_config(config);

    state.common_setup.complete_chain_config_setup_phase(None);
}
