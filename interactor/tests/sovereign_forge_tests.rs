use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait,
    constants::{ONE_HUNDRED_TOKENS, ONE_THOUSAND_TOKENS},
    interactor_config::Config,
};
use common_test_setup::constants::{CHAIN_ID, DEPLOY_COST};
use multiversx_sc::{
    imports::OptionalValue,
    types::{BigUint, EsdtTokenPayment},
};
use multiversx_sc_snippets::imports::{tokio, Bech32Address, StaticApi};
use rust_interact::sovereign_forge::sovereign_forge_interactor_main::SovereignForgeInteract;
use structs::{aliases::PaymentsVec, forge::ScArray};

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
async fn test_deploy_sovereign_forge_cs() {
    let mut interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let deploy_cost = BigUint::from(DEPLOY_COST);

    interactor.deploy_sovereign_forge(&deploy_cost).await;
    let sovereign_forge_address = interactor
        .state
        .current_sovereign_forge_sc_address()
        .clone();

    interactor.deploy_chain_config(OptionalValue::None).await;
    let chain_config_address = interactor.state.current_chain_config_sc_address().clone();
    let contracts_array =
        interactor.get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    interactor.deploy_mvx_esdt_safe(OptionalValue::None).await;
    let mvx_esdt_safe_address = interactor
        .state
        .current_mvx_esdt_safe_contract_address()
        .clone();

    interactor
        .deploy_fee_market(mvx_esdt_safe_address.clone(), None)
        .await;
    let fee_market_address = interactor.state.current_fee_market_address().clone();

    interactor.deploy_header_verifier(contracts_array).await;
    let header_verifier_address = interactor.state.current_header_verifier_address().clone();

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
        .deploy_phase_one(deploy_cost, Some(CHAIN_ID.into()), OptionalValue::None)
        .await;
    interactor.deploy_phase_two(OptionalValue::None).await;
    interactor.deploy_phase_three(None).await;
    interactor.deploy_phase_four().await;

    interactor.complete_setup_phase().await;
    interactor.check_setup_phase_status(CHAIN_ID, true).await;
}

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Run deploy phases 1–4 and call complete_setup_phase
///
/// ### EXPECTED
/// Deposit is successful and tokens are transferred to the mvx-esdt-safe-sc
#[tokio::test]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_deposit_flow() {
    let mut interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let deploy_cost = BigUint::from(DEPLOY_COST);
    let user_address = interactor.user_address().clone();

    interactor
        .deploy_and_complete_setup_phase(
            deploy_cost,
            OptionalValue::None,
            OptionalValue::None,
            vec![ScArray::ChainConfig, ScArray::ESDTSafe],
            None,
        )
        .await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        interactor.state.get_first_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        interactor.state.get_second_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    interactor
        .deposit_mvx_esdt_safe(
            user_address,
            OptionalValue::None,
            payments_vec,
            None,
            Some("deposit"),
        )
        .await;

    let expected_tokens_wallet = vec![
        interactor.custom_amount_tokens(
            interactor.state.get_first_token_id_string(),
            ONE_THOUSAND_TOKENS - ONE_HUNDRED_TOKENS,
        ),
        interactor.custom_amount_tokens(
            interactor.state.get_second_token_id_string(),
            ONE_THOUSAND_TOKENS - ONE_HUNDRED_TOKENS,
        ),
    ];
    interactor
        .check_address_balance(
            &Bech32Address::from(interactor.wallet_address().clone()),
            expected_tokens_wallet,
        )
        .await;

    let expected_tokens_contract = vec![
        interactor.custom_amount_tokens(
            interactor.state.get_first_token_id_string(),
            ONE_HUNDRED_TOKENS,
        ),
        interactor.custom_amount_tokens(
            interactor.state.get_second_token_id_string(),
            ONE_HUNDRED_TOKENS,
        ),
    ];
    interactor
        .check_address_balance(
            &interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone(),
            expected_tokens_contract,
        )
        .await;
}
