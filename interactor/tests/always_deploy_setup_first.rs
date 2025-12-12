use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_test_setup::constants::DEPLOY_COST;
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_snippets::imports::tokio;
use rust_interact::mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use serial_test::serial;
use structs::configs::SovereignConfig;

/// ### SETUP
/// DEPLOY_CONTRACTS
///
/// ### ACTION
/// Deploys and completes setup phases for all smart contracts
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deploy_setup() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    chain_interactor
        .deploy_and_complete_setup_phase(
            OptionalValue::Some(DEPLOY_COST.into()),
            OptionalValue::Some(SovereignConfig::default_config_for_test()),
            OptionalValue::None,
        )
        .await;
}
