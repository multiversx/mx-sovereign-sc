pub mod enshrine_esdt_safe;
pub mod mvx_esdt_safe;
pub mod sovereign_forge;

use common_interactor::{
    common_sovereign_interactor::{CommonInteractorTrait, TemplateAddresses},
    interactor_config::Config,
};
use common_test_setup::constants::PREFERRED_CHAIN_IDS;
use enshrine_esdt_safe::enshrine_esdt_safe_interactor::EnshrineEsdtSafeInteract;
use multiversx_sc::{
    imports::{MultiValueVec, OptionalValue},
    types::{Address, BigUint, ManagedBuffer, ManagedVec},
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
        "deployChainConfig" => {
            interact
                .deploy_chain_config(
                    Address::zero(),
                    PREFERRED_CHAIN_IDS[0].to_string(),
                    OptionalValue::None,
                )
                .await
        }
        "deployHeaderVerifier" => {
            interact
                .deploy_header_verifier(Address::zero(), PREFERRED_CHAIN_IDS[0].to_string(), vec![])
                .await
        }
        "deployEsdtSafe" => {
            interact
                .deploy_mvx_esdt_safe(
                    Address::zero(),
                    PREFERRED_CHAIN_IDS[0].to_string(),
                    OptionalValue::None,
                )
                .await;
        }
        "deployFeeMarket" => {
            interact
                .deploy_fee_market(
                    Address::zero(),
                    PREFERRED_CHAIN_IDS[0].to_string(),
                    interact
                        .state
                        .current_mvx_esdt_safe_contract_address()
                        .clone(),
                    None,
                )
                .await;
        }
        "deployTestingSc" => {
            interact
                .deploy_testing_sc(Address::zero(), PREFERRED_CHAIN_IDS[0].to_string())
                .await;
        }
        "completeSetup" => interact.complete_setup_phase().await,
        "completeHeaderVerifierSetup" => {
            interact
                .complete_header_verifier_setup_phase(Address::zero())
                .await
        }
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
        "upgrade" => interact.upgrade(Address::zero()).await,
        "deploySovereignForge" => {
            interact
                .deploy_sovereign_forge(Address::zero(), &BigUint::from(100u64))
                .await;
        }
        "deployChainFactory" => {
            interact
                .deploy_chain_factory(
                    Address::zero(),
                    PREFERRED_CHAIN_IDS[0].to_string(),
                    interact
                        .state
                        .current_sovereign_forge_sc_address()
                        .to_address(),
                    TemplateAddresses {
                        chain_config_address: interact
                            .state
                            .current_chain_config_sc_address()
                            .clone(),

                        header_verifier_address: interact
                            .state
                            .current_header_verifier_address()
                            .clone(),

                        esdt_safe_address: interact
                            .state
                            .current_mvx_esdt_safe_contract_address()
                            .clone(),

                        fee_market_address: interact.state.current_fee_market_address().clone(),
                    },
                )
                .await;
        }
        "deployChainConfig" => {
            interact
                .deploy_chain_config(
                    Address::zero(),
                    PREFERRED_CHAIN_IDS[0].to_string(),
                    OptionalValue::None,
                )
                .await
        }
        "deployHeaderVerifier" => {
            interact
                .deploy_header_verifier(Address::zero(), PREFERRED_CHAIN_IDS[0].to_string(), vec![])
                .await
        }
        "deployEsdtSafe" => {
            interact
                .deploy_mvx_esdt_safe(
                    Address::zero(),
                    PREFERRED_CHAIN_IDS[0].to_string(),
                    OptionalValue::None,
                )
                .await;
        }
        "deployFeeMarket" => {
            interact
                .deploy_fee_market(
                    Address::zero(),
                    PREFERRED_CHAIN_IDS[0].to_string(),
                    interact
                        .state
                        .current_mvx_esdt_safe_contract_address()
                        .clone(),
                    None,
                )
                .await;
        }
        "registerTokenHandler" => {
            interact
                .register_token_handler(Address::zero(), 0, PREFERRED_CHAIN_IDS[0].to_string())
                .await
        }
        "registerChainFactory" => {
            interact
                .register_chain_factory(Address::zero(), 0, PREFERRED_CHAIN_IDS[0].to_string())
                .await
        }
        "completeSetup" => interact.complete_setup_phase(Address::zero()).await,
        "deployPhaseOne" => {
            interact
                .deploy_phase_one(
                    Address::zero(),
                    BigUint::from(100u64),
                    None,
                    OptionalValue::None,
                )
                .await
        }
        "deployPhaseTwo" => {
            interact
                .deploy_phase_two(Address::zero(), OptionalValue::None)
                .await
        }
        "deployPhaseThree" => interact.deploy_phase_three(Address::zero(), None).await,
        "deployPhaseFour" => interact.deploy_phase_four(Address::zero()).await,
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
                    Address::zero(),
                    PREFERRED_CHAIN_IDS[0].to_string(),
                    false,
                    None,
                    None,
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
                    interact.user_address.clone(),
                    OptionalValue::None,
                    None,
                    None,
                )
                .await
        }
        "executeBridgeOps" => {
            interact
                .execute_operation(
                    &ManagedBuffer::new(),
                    Operation::new(
                        interact.user_address.clone().into(),
                        ManagedVec::new(),
                        OperationData::new(0, interact.user_address.clone().into(), None),
                    ),
                    None,
                    None,
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
