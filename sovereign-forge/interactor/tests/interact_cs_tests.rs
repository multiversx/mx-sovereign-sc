use forge_rust_interact::ContractInteract;
use multiversx_sc_snippets::imports::tokio;

#[tokio::test]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deploy_test_sovereign_forge_cs() {
    let mut interactor = ContractInteract::new().await;
    interactor.deploy().await;
    interactor.deploy_chain_config_template().await;
    interactor.deploy_header_verifier_template().await;
    interactor.deploy_mvx_esdt_safe_template().await;
    interactor.deploy_fee_market_template().await;
    interactor.deploy_chain_factory().await;

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
