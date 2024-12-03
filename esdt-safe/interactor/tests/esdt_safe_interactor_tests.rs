use esdt_safe_interactor::ContractInteract;
use interactor::interactor_config::Config;
use multiversx_sc_scenario::imports::*;
use multiversx_sc_scenario::scenario_model::TxResponseStatus;
use multiversx_sc_snippets::imports::*;

#[tokio::test]
#[ignore]
async fn test_deploy_sov() {
    let mut interact = ContractInteract::new(Config::load_config()).await;
    interact.deploy(false).await;
    interact.deploy_fee_market().await;
    interact.set_fee_market_address().await;
    interact.remove_fee().await;
    interact.deploy_header_verifier_contract().await;
    interact.set_header_verifier_address().await;
    interact.unpause_endpoint().await;
    interact.header_verifier_set_esdt_address().await;
    interact.deploy_testing_contract().await;
    interact.register_token().await;

    let operation = interact.setup_operation(true).await;
    interact.register_operations(&operation).await;
    interact
        .execute_operations(
            &operation,
            Some(TxResponseStatus::new(
                ReturnCode::UserError,
                "Value should be greater than 0",
            )),
        )
        .await;
}
