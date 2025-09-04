use chain_factory_blackbox_setup::ChainFactoryTestState;
use multiversx_sc::imports::OptionalValue;

mod chain_factory_blackbox_setup;

#[test]
fn test_deploy() {
    let mut state = ChainFactoryTestState::new();
    state.common_setup.deploy_chain_factory();
}

/// ### TEST
/// C-FACTORY_DEPLOY_CHAIN_CONFIG_OK
///
/// ### ACTION
/// Call 'deploy_chain_config_from_factory()' with a valid config
///
/// ### EXPECTED
/// Chain config is deployed correctly
#[test]
fn test_deploy_chain_config_from_factory() {
    let mut state = ChainFactoryTestState::new();

    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);
    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state.common_setup.deploy_chain_factory();

    state.deploy_chain_config_from_factory(OptionalValue::None, None);
}
