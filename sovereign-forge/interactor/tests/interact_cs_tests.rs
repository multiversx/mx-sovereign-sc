use forge_rust_interact::ContractInteract;
use multiversx_sc_snippets::imports::*;

// Simple deploy test that runs using the chain simulator configuration.
// In order for this test to work, make sure that the `config.toml` file contains the chain simulator config (or choose it manually)
// The chain simulator should already be installed and running before attempting to run this test.
// The chain-simulator-tests feature should be present in Cargo.toml.
// Can be run with `sc-meta test -c`.
#[tokio::test]
// #[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deploy_test_sovereign_forge_cs() {
    let mut interactor = ContractInteract::new().await;

    interactor.deploy().await;
    println!("=========CHAIN FACTORY DEPLOY==========");
    interactor.deploy_chain_factory().await;
    interactor.deploy_chain_config_template().await;
    println!("=========HEADER VERIFIER DEPLOY==========");
    interactor.deploy_header_verifier_template().await;

    println!("=========REGISTER TOKEN HANDLERS==========");
    interactor.register_token_handler(1).await;
    interactor.register_token_handler(2).await;
    interactor.register_token_handler(3).await;

    println!("=========REGISTER CHAIN FACTORIES==========");
    interactor.register_chain_factory(1).await;
    interactor.register_chain_factory(2).await;
    interactor.register_chain_factory(3).await;

    println!("=========COMPLETE SETUP PHASE==========");
    interactor.complete_setup_phase().await;

    println!("=========PHASE ONE START==========");
    interactor.deploy_phase_one().await;
    println!("=========PHASE ONE END ==========");

    interactor.deploy_phase_two().await;
    println!("=========PHASE TWO==========");

    interactor.deploy_phase_three().await;
    println!("=========PHASE THREE==========");
}
