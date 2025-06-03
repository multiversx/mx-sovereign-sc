use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use common_test_setup::constants::CHAIN_ID;
use multiversx_sc::{imports::OptionalValue, types::BigUint};
use multiversx_sc_snippets::imports::tokio;
use rust_interact::sovereign_forge::sovereign_forge_interactor_main::SovereignForgeInteract;
use structs::configs::SovereignConfig;

/// ### TEST
/// S-FORGE_COMPLETE_SETUP_PHASE_OK
///
/// ### ACTION
/// Run deploy phases 1–4 and call complete_setup_phase
///
/// ### EXPECTED
/// Setup phase is complete
#[tokio::test]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deploy_test_sovereign_forge_cs() {
    let mut interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let deploy_cost = BigUint::from(100u32);

    interactor.deploy_sovereign_forge(&deploy_cost).await;
    let sovereign_forge_address = interactor
        .state
        .current_sovereign_forge_sc_address()
        .clone();

    interactor
        .deploy_chain_config(SovereignConfig::default_config())
        .await;
    let chain_config_address = interactor.state.current_chain_config_sc_address().clone();

    interactor
        .deploy_header_verifier(chain_config_address.clone())
        .await;
    let header_verifier_address = interactor.state.current_header_verifier_address().clone();

    interactor
        .deploy_mvx_esdt_safe(header_verifier_address.clone(), OptionalValue::None)
        .await;
    let mvx_esdt_safe_address = interactor
        .state
        .current_mvx_esdt_safe_contract_address()
        .clone();

    interactor
        .deploy_fee_market(mvx_esdt_safe_address.clone(), None)
        .await;
    let fee_market_address = interactor.state.current_fee_market_address().clone();

    interactor
        .deploy_chain_factory(
            sovereign_forge_address,
            chain_config_address,
            header_verifier_address,
            mvx_esdt_safe_address,
            fee_market_address,
        )
        .await;
    let chain_factory_address = interactor.state.current_chain_factory_sc_address().clone();

    interactor
        .deploy_token_handler(chain_factory_address.to_address())
        .await;

    interactor.register_token_handler(1).await;
    interactor.register_token_handler(2).await;
    interactor.register_token_handler(3).await;
    interactor.register_chain_factory(1).await;
    interactor.register_chain_factory(2).await;
    interactor.register_chain_factory(3).await;

    interactor
        .deploy_phase_one(
            deploy_cost,
            Some(CHAIN_ID.into()),
            SovereignConfig::default_config(),
        )
        .await;
    interactor.deploy_phase_two().await;
    interactor.deploy_phase_three(OptionalValue::None).await;
    interactor.deploy_phase_four(None).await;

    interactor.complete_setup_phase().await;
    interactor.check_setup_phase_status(CHAIN_ID, true).await;
}
