use common_interactor::{
    common_interactor_sovereign::CommonInteractorTrait, interactor_config::Config,
};
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_snippets::imports::tokio;
use rust_interact::sovereign_forge::sovereign_forge_interactor_main::SovereignForgeInteract;

#[tokio::test]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deploy_test_sovereign_forge_cs() {
    let mut interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    interactor.deploy_sovereign_forge().await;
    interactor.deploy_chain_config().await;
    interactor.deploy_header_verifier().await;
    interactor.deploy_mvx_esdt_safe(OptionalValue::None).await;
    interactor.deploy_fee_market(None).await;
    interactor.deploy_chain_factory().await;
    interactor.deploy_token_handler().await;

    interactor.register_token_handler(1).await;
    interactor.register_token_handler(2).await;
    interactor.register_token_handler(3).await;
    interactor.register_chain_factory(1).await;
    interactor.register_chain_factory(2).await;
    interactor.register_chain_factory(3).await;

    interactor.complete_setup_phase().await;
    interactor.deploy_phase_one().await;
    interactor.deploy_phase_two().await;
    interactor.deploy_phase_three().await;
    interactor.deploy_phase_four().await;
}
