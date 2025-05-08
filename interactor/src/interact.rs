pub mod enshrine_esdt_safe;
pub mod mvx_esdt_safe;
pub mod sovereign_forge;

use common_interactor::{
    common_interactor_sovereign::CommonInteractorTrait, constants::TOKEN_ID,
    interactor_config::Config,
};
use enshrine_esdt_safe::enshrine_esdt_safe_interactor::EnshrineEsdtSafeInteract;
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_snippets::{env_logger, imports::StaticApi};
use mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use proxies::fee_market_proxy::FeeStruct;
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
        "deployChainConfig" => interact.deploy_chain_config().await,
        "deployHeaderVerifier" => interact.deploy_header_verifier().await,
        "deployEsdtSafe" => {
            let config = OptionalValue::None;
            interact.deploy_mvx_esdt_safe(config).await;
        }
        "deployFeeMarket" => {
            let fee: Option<FeeStruct<StaticApi>> = None;
            interact.deploy_fee_market(fee).await;
        }
        "deployTestingSc" => interact.deploy_testing_sc().await,
        "completeSetup" => interact.complete_setup_phase().await,
        "completeHeaderVerifierSetup" => interact.complete_header_verifier_setup_phase().await,
        "setEsdtInVerifier" => interact.set_esdt_safe_address_in_header_verifier().await,
        "resetState" => interact.reset_state_chain_sim(None).await,
        "resetStateTokens" => interact.reset_state_chain_sim_register_tokens().await,
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
        "deploySovereignForge" => interact.deploy_sovereign_forge().await,
        "deployChainFactory" => interact.deploy_chain_factory().await,
        "deployChainConfig" => interact.deploy_chain_config().await,
        "deployHeaderVerifier" => interact.deploy_header_verifier().await,
        "deployEsdtSafe" => interact.deploy_mvx_esdt_safe(OptionalValue::None).await,
        "deployFeeMarket" => interact.deploy_fee_market(None).await,
        "registerTokenHandler" => interact.register_token_handler(0).await,
        "registerChainFactory" => interact.register_chain_factory(0).await,
        "completeSetup" => interact.complete_setup_phase().await,
        "deployPhaseOne" => interact.deploy_phase_one().await,
        "deployPhaseTwo" => interact.deploy_phase_two().await,
        "deployPhaseThree" => interact.deploy_phase_three().await,
        "deployPhaseFour" => interact.deploy_phase_four().await,
        "getChainFactories" => interact.get_chain_factories().await,
        "getTokenHandlers" => interact.get_token_handlers().await,
        "getDeployCost" => interact.get_deploy_cost().await,
        "getChainIds" => interact.get_chain_ids().await,
        _ => panic!("Unknown command: {}", cmd),
    }
}

pub async fn enshrine_esdt_safe_cli() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");

    let config = Config::load_config();
    let mut interact = EnshrineEsdtSafeInteract::new(config).await;
    match cmd.as_str() {
        "deploy" => interact.deploy_enshrine_esdt(false, None).await,
        "upgrade" => interact.upgrade().await,
        "setFeeMarketAddress" => {
            interact
                .set_fee_market_address_in_enshrine_esdt_safe()
                .await
        }
        "setHeaderVerifierAddress" => {
            interact
                .set_header_verifier_address_in_enshrine_esdt_safe()
                .await
        }
        "deposit" => interact.deposit(None.into(), Option::None).await,
        "executeBridgeOps" => interact.execute_operations().await,
        "registerNewTokenID" => interact.register_new_token_id().await,
        "addTokensToWhitelist" => interact.add_tokens_to_whitelist(TOKEN_ID).await,
        "removeTokensFromWhitelist" => interact.remove_tokens_from_whitelist().await,
        "addTokensToBlacklist" => interact.add_tokens_to_blacklist().await,
        "removeTokensFromBlacklist" => interact.remove_tokens_from_blacklist().await,
        "getTokenWhitelist" => interact.token_whitelist().await,
        "getTokenBlacklist" => interact.token_blacklist().await,
        "pause" => interact.pause_endpoint().await,
        "unpause" => interact.unpause_endpoint().await,
        "isPaused" => interact.paused_status().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}
