use forge_rust_interact::ContractInteract;
use multiversx_sc_snippets::imports::tokio;

#[tokio::test]
#[ignore = "run on demand, relies on real blockchain state"]
async fn deploy_test_sovereign_forge() {
    let mut interactor = ContractInteract::new().await;
    interactor.deploy().await;

    interactor.deploy_chain_factory().await;
    interactor.deploy_chain_config_template().await;
    interactor.deploy_header_verifier_template().await;

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
}
