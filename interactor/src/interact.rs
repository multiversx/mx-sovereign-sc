pub mod mvx_esdt_safe;
pub mod sovereign_forge;

use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_snippets::env_logger;
use mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use sovereign_forge::sovereign_forge_interactor_main::SovereignForgeInteract;

pub async fn mvx_esdt_safe_cli() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let config = Config::load_config();
    let mut interact = MvxEsdtSafeInteract::new(config).await;
    match cmd.as_str() {
        "upgrade" => interact.upgrade().await,
        "pause" => interact.pause_endpoint().await,
        "unpause" => interact.unpause_endpoint().await,
        "isPaused" => interact.paused_status().await,
        "deployChainConfig" => {
            interact.deploy_chain_config(OptionalValue::None).await;
        }
        "deployHeaderVerifier" => {
            interact.deploy_header_verifier(vec![]).await;
        }
        "deployEsdtSafe" => {
            interact.deploy_mvx_esdt_safe(OptionalValue::None).await;
        }
        "deployFeeMarket" => {
            interact
                .deploy_fee_market(
                    interact
                        .state
                        .current_mvx_esdt_safe_contract_address()
                        .clone(),
                    None,
                )
                .await;
        }
        "deployTestingSc" => interact.deploy_testing_sc().await,
        "completeSetup" => interact.complete_setup_phase().await,
        "completeHeaderVerifierSetup" => interact.complete_header_verifier_setup_phase().await,
        _ => panic!("Unknown command: {}", cmd),
    }
}

pub async fn sovereign_forge_cli() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let config = Config::load_config();

    let mut interact = SovereignForgeInteract::new(config).await;

    match cmd.as_str() {
        "upgrade" => interact.upgrade().await,
        "deploySovereignForge" => interact.deploy_sovereign_forge(OptionalValue::None).await,
        "deployChainFactory" => {
            interact
                .deploy_chain_factory(
                    interact.state.current_sovereign_forge_sc_address().clone(),
                    interact.state.current_chain_config_sc_address().clone(),
                    interact.state.current_header_verifier_address().clone(),
                    interact
                        .state
                        .current_mvx_esdt_safe_contract_address()
                        .clone(),
                    interact.state.current_fee_market_address().clone(),
                    0,
                )
                .await
        }
        "deployChainConfig" => {
            interact.deploy_chain_config(OptionalValue::None).await;
        }
        "deployHeaderVerifier" => {
            interact.deploy_header_verifier(vec![]).await;
        }
        "deployEsdtSafe" => {
            interact.deploy_mvx_esdt_safe(OptionalValue::None).await;
        }
        "deployFeeMarket" => {
            interact
                .deploy_fee_market(
                    interact
                        .state
                        .current_mvx_esdt_safe_contract_address()
                        .clone(),
                    None,
                )
                .await;
        }
        "registerChainFactory" => interact.register_chain_factory(0).await,
        "completeSetup" => interact.complete_setup_phase().await,
        "deployPhaseOne" => {
            interact
                .deploy_phase_one(OptionalValue::None, None, OptionalValue::None)
                .await
        }
        "deployPhaseTwo" => interact.deploy_phase_two(OptionalValue::None).await,
        "deployPhaseThree" => interact.deploy_phase_three(None).await,
        "deployPhaseFour" => interact.deploy_phase_four().await,
        "getChainFactories" => interact.get_chain_factories().await,
        "getDeployCost" => interact.get_deploy_cost().await,
        "getChainIds" => interact.get_chain_ids().await,
        _ => panic!("Unknown command: {}", cmd),
    }
}
