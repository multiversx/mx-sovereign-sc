use chain_factory_blackbox_setup::ChainFactoryTestState;
use multiversx_sc::types::BigUint;
use structs::configs::SovereignConfig;

mod chain_factory_blackbox_setup;

#[test]
fn test_deploy() {
    let mut state = ChainFactoryTestState::new();
    state.common_setup.deploy_chain_factory();
}

/// ### TEST
/// C-FACTORY_DEPLOY_CHAIN_CONFIG_OK_001
///
/// ### ACTION
/// Call 'deploy_chain_config_from_factory()' with a valid config
///
/// ### EXPECTED
/// Chain config is deployed correctly
#[test]
fn test_deploy_chain_config_from_factory() {
    let mut state = ChainFactoryTestState::new();

    state.common_setup.deploy_sovereign_forge();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state.common_setup.deploy_chain_factory();

    let config = SovereignConfig::new(0, 1, BigUint::default(), None);

    state.deploy_chain_config_from_factory(config, None);
}
