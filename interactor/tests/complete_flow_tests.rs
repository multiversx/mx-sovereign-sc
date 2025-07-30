use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_interactor::interactor_state::EsdtTokenInfo;
use common_test_setup::base_setup::init::RegisterTokenArgs;
use common_test_setup::constants::GAS_LIMIT;
use common_test_setup::constants::PER_GAS;
use common_test_setup::constants::PER_TRANSFER;
use common_test_setup::constants::{
    DEPLOY_COST, ISSUE_COST, ONE_HUNDRED_TOKENS, REGISTER_TOKEN_PREFIX, SC_CALL_LOG, SHARD_1,
    SHARD_2, TESTING_SC_ENDPOINT, TOKEN_DISPLAY_NAME, TOKEN_TICKER, WRONG_ENDPOINT_NAME,
};
use multiversx_sc::imports::OptionalValue;
use multiversx_sc::types::EsdtTokenType;
use multiversx_sc::types::{BigUint, TokenIdentifier};
use multiversx_sc_snippets::imports::{tokio, StaticApi};
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::vm_err_msg::FUNCTION_NOT_FOUND;
use rstest::rstest;
use rust_interact::sovereign_forge::sovereign_forge_interactor_main::SovereignForgeInteract;
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .complete_deposit_flow_with_transfer_data_only(shard, None, Some(SC_CALL_LOG))
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;
    chain_interactor.check_testing_sc_balance_is_empty().await;
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    let fee = chain_interactor.create_standard_fee();
    let fee_token = chain_interactor.state.get_fee_token_id();
    let fee_amount = BigUint::from(PER_GAS * GAS_LIMIT);

    chain_interactor
        .complete_deposit_flow_with_transfer_data_only(shard, Some(fee), Some(SC_CALL_LOG))
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;

    chain_interactor
        .check_fee_market_balance_with_amount(shard, fee_token.clone(), fee_amount.clone())
        .await;

    chain_interactor
        .check_user_balance_after_deduction(fee_token, fee_amount)
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .complete_execute_operation_flow_with_transfer_data_only(
            shard,
            None,
            Some(""),
            None,
            TESTING_SC_ENDPOINT,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;
    chain_interactor.check_testing_sc_balance_is_empty().await;
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    //NOTE: For now, there is a failed log only for top_encode error, which is hard to achieve. If the sc returns an error, the logs are no longer retrieved by the framework
    chain_interactor
        .complete_execute_operation_flow_with_transfer_data_only(
            shard,
            Some(FUNCTION_NOT_FOUND),
            None,
            None,
            WRONG_ENDPOINT_NAME,
        )
        .await;
    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;
    chain_interactor.check_testing_sc_balance_is_empty().await;
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    let token = chain_interactor.get_token_by_type(token_type);
    let fee_amount = BigUint::from(PER_TRANSFER);

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
        .deposit_no_transfer_data(shard, token.clone(), amount.clone(), Some(fee))
        .await;

    chain_interactor
        .check_user_balance_with_fee_deduction(token.clone(), amount.clone(), fee_amount.clone())
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_with_amount(shard, token, amount)
        .await;

    chain_interactor
        .check_fee_market_balance_with_amount(
            shard,
            chain_interactor.state.get_fee_token_id(),
            fee_amount,
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

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
        .deposit_no_transfer_data(shard, token.clone(), amount.clone(), None)
        .await;

    chain_interactor
        .check_user_balance_after_deduction(token.clone(), amount.clone())
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_with_amount(shard, token.clone(), amount.clone())
        .await;

    chain_interactor
        .execute_operation(
            shard,
            None,
            Some(&token.token_id),
            token.clone(),
            amount.clone(),
            None,
        )
        .await;

    chain_interactor
        .check_user_balance_with_amount(token.clone(), token.amount.clone())
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;

    chain_interactor
        .check_fee_market_balance_is_empty(shard)
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let token_id = "SOV-123456";
    let sov_token_id =
        TokenIdentifier::from_esdt_bytes(REGISTER_TOKEN_PREFIX.to_string() + token_id);
    let token_display_name = TOKEN_DISPLAY_NAME;
    let num_decimals = decimals;
    let wanted_token_id = token_id;
    let token_ticker = wanted_token_id.split('-').next().unwrap_or(TOKEN_TICKER);

    chain_interactor
        .register_token(
            shard,
            RegisterTokenArgs {
                sov_token_id: sov_token_id.clone(),
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            ISSUE_COST.into(),
            None,
        )
        .await;

    let expected_token = chain_interactor
        .get_sov_to_mvx_token_id(shard, sov_token_id.clone())
        .await;

    let token_info = EsdtTokenInfo {
        token_id: sov_token_id.to_string(),
        nonce,
        token_type,
        amount: amount.clone(),
    };

    chain_interactor
        .execute_operation(
            shard,
            None,
            Some(&expected_token.to_string()),
            token_info,
            amount.clone(),
            None,
        )
        .await;

    let expected_token_info = EsdtTokenInfo {
        token_id: expected_token.to_string(),
        nonce,
        token_type,
        amount: amount.clone(),
    };

    chain_interactor
        .check_user_balance_with_amount(
            expected_token_info.clone(),
            expected_token_info.amount.clone(),
        )
        .await;

    if token_type == EsdtTokenType::MetaFungible
        || token_type == EsdtTokenType::DynamicMeta
        || token_type == EsdtTokenType::DynamicSFT
        || token_type == EsdtTokenType::SemiFungible
    {
        chain_interactor
            .check_mvx_esdt_safe_balance_with_amount(
                shard,
                expected_token_info.clone(),
                1u64.into(),
            )
            .await;
    } else {
        chain_interactor
            .check_mvx_esdt_safe_balance_is_empty(shard)
            .await;
    }

    chain_interactor
        .deposit_no_transfer_data(shard, expected_token_info.clone(), amount.clone(), None)
        .await;

    chain_interactor
        .check_user_balance_after_deduction(expected_token_info.clone(), amount)
        .await;

    if token_type == EsdtTokenType::MetaFungible
        || token_type == EsdtTokenType::DynamicMeta
        || token_type == EsdtTokenType::DynamicSFT
        || token_type == EsdtTokenType::SemiFungible
    {
        chain_interactor
            .check_mvx_esdt_safe_balance_with_amount(
                shard,
                expected_token_info.clone(),
                1u64.into(),
            )
            .await;
    } else {
        chain_interactor
            .check_mvx_esdt_safe_balance_is_empty(shard)
            .await;
    }
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

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
        .deposit_with_transfer_data(shard, token.clone(), amount.clone(), None)
        .await;

    chain_interactor
        .check_user_balance_after_deduction(token.clone(), amount.clone())
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_with_amount(shard, token.clone(), amount.clone())
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    let fee = chain_interactor.create_standard_fee();
    let fee_token = chain_interactor.state.get_fee_token_id();
    let fee_amount = BigUint::from(PER_TRANSFER + PER_GAS * GAS_LIMIT);

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
        .deposit_with_transfer_data(shard, token.clone(), amount.clone(), Some(fee))
        .await;

    chain_interactor
        .check_user_balance_with_fee_deduction(token.clone(), amount.clone(), fee_amount.clone())
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_with_amount(shard, token.clone(), amount.clone())
        .await;

    chain_interactor
        .check_fee_market_balance_with_amount(shard, fee_token, fee_amount)
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

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
        .deposit_no_transfer_data(shard, token.clone(), amount.clone(), None)
        .await;

    chain_interactor
        .execute_operation(
            shard,
            None,
            Some(&token.token_id),
            token.clone(),
            amount.clone(),
            Some(TESTING_SC_ENDPOINT),
        )
        .await;

    chain_interactor
        .check_user_balance_after_deduction(token.clone(), amount.clone())
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;

    chain_interactor
        .check_testing_sc_balance_with_amount(token.clone(), amount.clone())
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let token_id = "SOV-123456";

    let sov_token_id =
        TokenIdentifier::from_esdt_bytes(REGISTER_TOKEN_PREFIX.to_string() + token_id);
    let token_display_name = TOKEN_DISPLAY_NAME;
    let num_decimals = decimals;
    let wanted_token_id = token_id;
    let token_ticker = wanted_token_id.split('-').next().unwrap_or(TOKEN_TICKER);

    chain_interactor
        .register_token(
            shard,
            RegisterTokenArgs {
                sov_token_id: sov_token_id.clone(),
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            ISSUE_COST.into(),
            None,
        )
        .await;

    let expected_token = chain_interactor
        .get_sov_to_mvx_token_id(shard, sov_token_id.clone())
        .await;

    let token_info = EsdtTokenInfo {
        token_id: sov_token_id.to_string(),
        nonce,
        token_type,
        amount: amount.clone(),
    };

    chain_interactor
        .execute_operation(
            shard,
            None,
            Some(&expected_token.to_string()),
            token_info,
            amount.clone(),
            Some(TESTING_SC_ENDPOINT),
        )
        .await;

    let expected_token_info = EsdtTokenInfo {
        token_id: expected_token.to_string(),
        nonce,
        token_type,
        amount: amount.clone(),
    };

    chain_interactor.check_user_balance_unchanged().await;

    chain_interactor
        .check_testing_sc_balance_with_amount(expected_token_info.clone(), amount.clone())
        .await;

    if token_type == EsdtTokenType::MetaFungible
        || token_type == EsdtTokenType::DynamicMeta
        || token_type == EsdtTokenType::DynamicSFT
        || token_type == EsdtTokenType::SemiFungible
    {
        chain_interactor
            .check_mvx_esdt_safe_balance_with_amount(
                shard,
                expected_token_info.clone(),
                1u64.into(),
            )
            .await;
    } else {
        chain_interactor
            .check_mvx_esdt_safe_balance_is_empty(shard)
            .await;
    }

    chain_interactor
        .withdraw_from_testing_sc(expected_token.clone(), nonce, amount.clone())
        .await;

    chain_interactor.check_testing_sc_balance_is_empty().await;

    chain_interactor
        .check_user_balance_with_amount(expected_token_info.clone(), amount.clone())
        .await;

    chain_interactor
        .deposit_no_transfer_data(shard, expected_token_info.clone(), amount, None)
        .await;

    chain_interactor
        .check_user_balance_with_amount(expected_token_info.clone(), 0u64.into())
        .await;

    if token_type == EsdtTokenType::MetaFungible
        || token_type == EsdtTokenType::DynamicMeta
        || token_type == EsdtTokenType::DynamicSFT
        || token_type == EsdtTokenType::SemiFungible
    {
        chain_interactor
            .check_mvx_esdt_safe_balance_with_amount(
                shard,
                expected_token_info.clone(),
                1u64.into(),
            )
            .await;
    } else {
        chain_interactor
            .check_mvx_esdt_safe_balance_is_empty(shard)
            .await;
    }
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
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let token_id = "SOV-123456";

    let sov_token_id =
        TokenIdentifier::from_esdt_bytes(REGISTER_TOKEN_PREFIX.to_string() + token_id);
    let token_display_name = TOKEN_DISPLAY_NAME;
    let num_decimals = decimals;
    let wanted_token_id = token_id;
    let token_ticker = wanted_token_id.split('-').next().unwrap_or(TOKEN_TICKER);

    chain_interactor
        .register_token(
            shard,
            RegisterTokenArgs {
                sov_token_id: sov_token_id.clone(),
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            ISSUE_COST.into(),
            None,
        )
        .await;

    let token_info = EsdtTokenInfo {
        token_id: sov_token_id.to_string(),
        nonce,
        token_type,
        amount: amount.clone(),
    };

    chain_interactor
        .execute_operation(
            shard,
            Some(FUNCTION_NOT_FOUND),
            None,
            token_info,
            amount.clone(),
            Some(WRONG_ENDPOINT_NAME),
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;

    let expected_token = chain_interactor
        .get_sov_to_mvx_token_id(shard, sov_token_id)
        .await;

    let expected_token_info = EsdtTokenInfo {
        token_id: expected_token.to_string(),
        nonce,
        token_type,
        amount: amount.clone(),
    };

    if token_type == EsdtTokenType::MetaFungible
        || token_type == EsdtTokenType::DynamicMeta
        || token_type == EsdtTokenType::DynamicSFT
        || token_type == EsdtTokenType::SemiFungible
    {
        chain_interactor
            .check_mvx_esdt_safe_balance_with_amount(
                shard,
                expected_token_info.clone(),
                1u64.into(),
            )
            .await;
    } else {
        chain_interactor
            .check_mvx_esdt_safe_balance_is_empty(shard)
            .await;
    }

    chain_interactor.check_testing_sc_balance_is_empty().await;
}
