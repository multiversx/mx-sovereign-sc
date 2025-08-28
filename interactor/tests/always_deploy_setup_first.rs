use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_test_setup::constants::DEPLOY_COST;
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_snippets::imports::tokio;
use rust_interact::mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use serial_test::serial;

/// ### SETUP
/// DEPLOY_CONTRACTS
///
/// ### ACTION
/// Deploys and completes setup phases for all smart contracts
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deploy_setup() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config(None)).await;
    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
        )
        .await;
}
