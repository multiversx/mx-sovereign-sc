pub mod enshrine_esdt_safe;
pub mod mvx_esdt_safe;
pub mod sovereign_forge;

use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use enshrine_esdt_safe::enshrine_esdt_safe_interactor::EnshrineEsdtSafeInteract;
use multiversx_sc::{
    imports::{MultiValueVec, OptionalValue},
    types::{BigUint, ManagedBuffer, ManagedVec, MultiValueEncoded, TokenIdentifier},
};
use multiversx_sc_snippets::env_logger;
use mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use sovereign_forge::sovereign_forge_interactor_main::SovereignForgeInteract;
use structs::{
    aliases::PaymentsVec,
    operation::{Operation, OperationData},
};

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
        "deployChainConfig" => interact.deploy_chain_config(OptionalValue::None).await,
        // "deployHeaderVerifier" => {
        //     interact
        //         .deploy_header_verifier(interact.state.current_chain_config_sc_address().clone())
        //         .await
        // }
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
        "deploySovereignForge" => {
            interact
                .deploy_sovereign_forge(&BigUint::from(100u64))
                .await
        }
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
                )
                .await
        }
        "deployChainConfig" => interact.deploy_chain_config(OptionalValue::None).await,
        // "deployHeaderVerifier" => {
        //     interact
        //         .deploy_header_verifier(interact.state.current_chain_config_sc_address().clone())
        //         .await
        // }
        "deployEsdtSafe" => interact.deploy_mvx_esdt_safe(OptionalValue::None).await,
        "deployFeeMarket" => {
            interact
                .deploy_fee_market(
                    interact
                        .state
                        .current_mvx_esdt_safe_contract_address()
                        .clone(),
                    None,
                )
                .await
        }
        "registerTokenHandler" => interact.register_token_handler(0).await,
        "registerChainFactory" => interact.register_chain_factory(0).await,
        "completeSetup" => interact.complete_setup_phase().await,
        "deployPhaseOne" => {
            interact
                .deploy_phase_one(BigUint::from(100u64), None, OptionalValue::None)
                .await
        }
        // "deployPhaseTwo" => interact.deploy_phase_two().await,
        // "deployPhaseThree" => interact.deploy_phase_three(OptionalValue::None).await,
        // "deployPhaseFour" => interact.deploy_phase_four(None).await,
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
        "deploy" => {
            interact
                .deploy_enshrine_esdt(
                    false,
                    None,
                    None,
                    interact.state.current_token_handler_address().clone(),
                    None,
                )
                .await
        }
        "upgrade" => interact.upgrade().await,
        "setFeeMarketAddress" => {
            interact
                .set_fee_market_address_in_enshrine_esdt_safe(
                    interact.state.current_fee_market_address().clone(),
                )
                .await
        }
        "deposit" => {
            interact
                .deposit(
                    PaymentsVec::new(),
                    interact.bob_address.clone().into(),
                    OptionalValue::None,
                    None,
                )
                .await
        }
        "executeBridgeOps" => {
            interact
                .execute_operations(
                    ManagedBuffer::new(),
                    Operation::new(
                        interact.bob_address.clone().into(),
                        ManagedVec::new(),
                        OperationData::new(0, interact.bob_address.clone().into(), None),
                    ),
                )
                .await
        }
        "registerNewTokenID" => {
            interact
                .register_new_token_id(
                    TokenIdentifier::from_esdt_bytes(""),
                    0u64,
                    BigUint::from(100u64),
                    MultiValueEncoded::new(),
                )
                .await
        }
        "addTokensToWhitelist" => interact.add_tokens_to_whitelist(MultiValueVec::new()).await,
        "removeTokensFromWhitelist" => {
            interact
                .remove_tokens_from_whitelist(MultiValueVec::new())
                .await
        }
        "addTokensToBlacklist" => interact.add_tokens_to_blacklist(MultiValueVec::new()).await,
        "removeTokensFromBlacklist" => {
            interact
                .remove_tokens_from_blacklist(MultiValueVec::new())
                .await
        }
        "getTokenWhitelist" => interact.token_whitelist().await,
        "getTokenBlacklist" => interact.token_blacklist().await,
        "pause" => interact.pause_endpoint().await,
        "unpause" => interact.unpause_endpoint().await,
        "isPaused" => interact.paused_status().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}
