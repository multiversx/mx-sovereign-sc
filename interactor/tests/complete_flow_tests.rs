use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_interactor::interactor_helpers::InteractorHelpers;
use common_interactor::interactor_structs::ActionConfig;
use common_test_setup::constants::{
    DEPLOY_COST, ONE_HUNDRED_TOKENS, SC_CALL_LOG, SHARD_1, SHARD_2, TESTING_SC_ENDPOINT,
    WRONG_ENDPOINT_NAME,
};
use multiversx_sc::imports::OptionalValue;
use multiversx_sc::types::BigUint;
use multiversx_sc::types::EsdtTokenType;
use multiversx_sc::types::TokenIdentifier;
use multiversx_sc_snippets::imports::{tokio, StaticApi};
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::vm_err_msg::FUNCTION_NOT_FOUND;
use rstest::rstest;
use rust_interact::complete_flows::complete_flows_interactor_main::CompleteFlowInteract;
use serial_test::serial;

//TODO: Change expected log to be DEPOSIT_LOG and EXECUTED_BRIDGE_LOG instead of "" when the framework fix is implemented

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call deposit with transfer data only
///
/// ### EXPECTED
/// Deposit is successful and the event is found in logs
#[rstest]
#[case::different_shard(SHARD_2)]
#[case::same_shard(SHARD_1)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_deposit_flow_no_fee_only_transfer_data(#[case] shard: u32) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(SC_CALL_LOG.to_string()),
            None,
            None,
            None,
        )
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call deposit with fee and transfer data only
///
/// ### EXPECTED
/// Deposit is successful and the event is found in logs
#[rstest]
#[case::different_shard(SHARD_2)]
#[case::same_shard(SHARD_1)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_deposit_flow_with_fee_only_transfer_data(#[case] shard: u32) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    let fee = chain_interactor.create_standard_fee();

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            Some(fee.clone()),
        )
        .await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(SC_CALL_LOG.to_string()),
            None,
            None,
            Some(fee),
        )
        .await;
}

//TODO: Fix the logs after framework fix is implemented, check for the TESTING_SC_ENDPOINT executed log as well
/// ### TEST
/// S-FORGE_COMPLETE-EXEC-FLOW_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[rstest]
#[case::different_shard(SHARD_2)]
#[case::same_shard(SHARD_1)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_execute_flow_with_transfer_data_only_success(#[case] shard: u32) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    chain_interactor
        .execute_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log("".to_string()),
            None,
            None,
        )
        .await;
}

//TODO: Fix the logs after framework fix is implemented
/// ### TEST
/// S-FORGE_COMPLETE-EXEC-FAIL
///
/// ### ACTION
/// Call 'execute_operation()' with invalid endpoint in operation
///
/// ### EXPECTED
/// The operation is not executed in the testing smart contract
#[rstest]
#[case::different_shard(SHARD_2)]
#[case::same_shard(SHARD_1)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_execute_flow_with_transfer_data_only_fail(#[case] shard: u32) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    //NOTE: For now, there is a failed log only for top_encode error, which is hard to achieve. If the sc returns an error, the logs are no longer retrieved by the framework
    chain_interactor
        .execute_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(WRONG_ENDPOINT_NAME.to_string())
                .expect_error(FUNCTION_NOT_FOUND.to_string()),
            None,
            None,
        )
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call deposit with fee set
///
/// ### EXPECTED
/// Deposit is successful and the event is found in logs
#[rstest]
#[case::fungible(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::non_fungible(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case::semi_fungible(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::meta_fungible(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case::dynamic_sft(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::dynamic_meta(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_with_fee(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    let token = chain_interactor.get_token_by_type(token_type);

    let fee = chain_interactor.create_standard_fee();

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            Some(fee.clone()),
        )
        .await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .expect_log(token.clone().token_id),
            Some(token),
            Some(amount),
            Some(fee),
        )
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call deposit without fee and execute operation
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract and the event is found in logs
#[rstest]
#[case::fungible(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::non_fungible(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case::semi_fungible(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::meta_fungible(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case::dynamic_sft(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::dynamic_meta(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_without_fee_and_execute(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    let token = chain_interactor.get_token_by_type(token_type);

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .expect_log(token.clone().token_id),
            Some(token.clone()),
            Some(amount.clone()),
            None,
        )
        .await;

    chain_interactor
        .execute_wrapper(
            ActionConfig::new()
                .shard(shard)
                .expect_log(token.clone().token_id),
            Some(token),
            Some(amount),
        )
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-EXECUTE-SOVEREIGN-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call register token, execute operation and deposit sov token
///
/// ### EXPECTED
/// The deposit is successful and the event is found in logs
#[rstest]
#[rstest]
#[case::fungible(
    EsdtTokenType::Fungible,
    BigUint::from(ONE_HUNDRED_TOKENS),
    0u64,
    18usize
)]
#[case::non_fungible(EsdtTokenType::NonFungibleV2, BigUint::from(1u64), 1u64, 0usize)]
#[case::semi_fungible(
    EsdtTokenType::SemiFungible,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    0usize
)]
#[case::meta_fungible(
    EsdtTokenType::MetaFungible,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    18usize
)]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT, BigUint::from(1u64), 1u64, 0usize)]
#[case::dynamic_sft(
    EsdtTokenType::DynamicSFT,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    0usize
)]
#[case::dynamic_meta(
    EsdtTokenType::DynamicMeta,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    18usize
)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_execute_and_deposit_sov_token(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
    #[case] nonce: u64,
    #[case] decimals: usize,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let sov_token = chain_interactor
        .register_and_execute_sovereign_token(
            ActionConfig::new()
                .shard(shard)
                .for_register(token_type, decimals, nonce),
            amount.clone(),
        )
        .await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_sovereign_token_id(TokenIdentifier::from_esdt_bytes(&sov_token.token_id))
                .expect_log(sov_token.clone().token_id),
            Some(sov_token),
            Some(amount),
            None,
        )
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call deposit without fee and transfer data
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract and the event is found in logs
#[rstest]
#[case::fungible(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::non_fungible(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case::semi_fungible(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::meta_fungible(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case::dynamic_sft(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::dynamic_meta(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_mvx_token_with_transfer_data(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let token = chain_interactor.get_token_by_type(token_type);

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(token.clone().token_id),
            Some(token),
            Some(amount),
            None,
        )
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call deposit with fee and transfer data
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract and the event is found in logs
#[rstest]
#[case::fungible(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::non_fungible(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case::semi_fungible(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::meta_fungible(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case::dynamic_sft(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::dynamic_meta(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_mvx_token_with_transfer_data_and_fee(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    let fee = chain_interactor.create_standard_fee();

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            Some(fee.clone()),
        )
        .await;

    let token = chain_interactor.get_token_by_type(token_type);

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(token.clone().token_id),
            Some(token),
            Some(amount),
            Some(fee),
        )
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call deposit without fee and execute operation with transfer data
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract and the event is found in logs
#[rstest]
#[case::fungible(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::non_fungible(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case::semi_fungible(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::meta_fungible(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case::dynamic_sft(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case::dynamic_meta(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_and_execute_with_transfer_data(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    let token = chain_interactor.get_token_by_type(token_type);

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .expect_log(token.clone().token_id),
            Some(token.clone()),
            Some(amount.clone()),
            None,
        )
        .await;

    chain_interactor
        .execute_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(token.clone().token_id),
            Some(token.clone()),
            Some(amount.clone()),
        )
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-REGISTER_EXECUTE-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call register, execute with transfer data and deposit sov token
///
/// ### EXPECTED
/// The deposit is successful and the event is found in logs
#[rstest]
#[case::fungible(
    EsdtTokenType::Fungible,
    BigUint::from(ONE_HUNDRED_TOKENS),
    0u64,
    18usize
)]
#[case::non_fungible(EsdtTokenType::NonFungibleV2, BigUint::from(1u64), 1u64, 0usize)]
#[case::semi_fungible(
    EsdtTokenType::SemiFungible,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    0usize
)]
#[case::meta_fungible(
    EsdtTokenType::MetaFungible,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    18usize
)]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT, BigUint::from(1u64), 1u64, 0usize)]
#[case::dynamic_sft(
    EsdtTokenType::DynamicSFT,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    0usize
)]
#[case::dynamic_meta(
    EsdtTokenType::DynamicMeta,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    18usize
)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_execute_with_transfer_data_and_deposit_sov_token(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
    #[case] nonce: u64,
    #[case] decimals: usize,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let sov_token = chain_interactor
        .register_and_execute_sovereign_token(
            ActionConfig::new()
                .shard(shard)
                .for_register(token_type, decimals, nonce)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string()),
            amount.clone(),
        )
        .await;

    chain_interactor
        .withdraw_from_testing_sc(sov_token.clone(), nonce, amount.clone())
        .await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_sovereign_token_id(TokenIdentifier::from_esdt_bytes(&sov_token.token_id))
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(sov_token.clone().token_id),
            Some(sov_token.clone()),
            Some(amount.clone()),
            None,
        )
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-REGISTER_EXECUTE-FLOW_FAIL
///
/// ### ACTION
/// Deploy and complete setup phase, then call register, execute with transfer data
///
/// ### EXPECTED
/// The operation is not executed in the testing smart contract and the failed event is found in logs
#[rstest]
#[case::fungible(
    EsdtTokenType::Fungible,
    BigUint::from(ONE_HUNDRED_TOKENS),
    0u64,
    18usize
)]
#[case::non_fungible(EsdtTokenType::NonFungibleV2, BigUint::from(1u64), 1u64, 0usize)]
#[case::semi_fungible(
    EsdtTokenType::SemiFungible,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    0usize
)]
#[case::meta_fungible(
    EsdtTokenType::MetaFungible,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    18usize
)]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT, BigUint::from(1u64), 1u64, 0usize)]
#[case::dynamic_sft(
    EsdtTokenType::DynamicSFT,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    0usize
)]
#[case::dynamic_meta(
    EsdtTokenType::DynamicMeta,
    BigUint::from(ONE_HUNDRED_TOKENS),
    1u64,
    18usize
)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_execute_call_failed(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
    #[case] nonce: u64,
    #[case] decimals: usize,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    chain_interactor
        .register_and_execute_sovereign_token(
            ActionConfig::new()
                .shard(shard)
                .for_register(token_type, decimals, nonce)
                .with_endpoint(WRONG_ENDPOINT_NAME.to_string())
                .expect_error(FUNCTION_NOT_FOUND.to_string()),
            amount,
        )
        .await;
}
