use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_interactor::interactor_helpers::InteractorHelpers;
use common_interactor::interactor_state::EsdtTokenInfo;
use common_interactor::interactor_structs::ActionConfig;
use common_test_setup::constants::READ_NATIVE_TOKEN_TESTING_SC_ENDPOINT;
use common_test_setup::constants::{
    DEPOSIT_LOG, ONE_HUNDRED_TOKENS, SC_CALL_LOG, SHARD_0, SHARD_1, SHARD_2, TESTING_SC_ENDPOINT,
    WRONG_ENDPOINT_NAME,
};
use multiversx_sc::types::BigUint;
use multiversx_sc::types::EgldOrEsdtTokenIdentifier;
use multiversx_sc::types::EsdtTokenType;
use multiversx_sc::types::ManagedAddress;
use multiversx_sc::types::ManagedBuffer;
use multiversx_sc::types::ManagedVec;
use multiversx_sc::types::MultiValueEncoded;
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_snippets::imports::{tokio, StaticApi};
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::vm_err_msg::FUNCTION_NOT_FOUND;
use rstest::rstest;
use rust_interact::complete_flows::complete_flows_interactor_main::CompleteFlowInteract;
use serial_test::serial;
use structs::operation::Operation;
use structs::operation::OperationData;
use structs::operation::TransferData;
use structs::OperationHashStatus;

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

    chain_interactor.remove_fee(shard).await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(vec![SC_CALL_LOG.to_string()]),
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

    chain_interactor.set_fee(fee.clone(), shard).await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(vec![SC_CALL_LOG.to_string()]),
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

    chain_interactor.remove_fee(shard).await;

    chain_interactor
        .execute_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(vec!["".to_string()]),
            None,
        )
        .await;
}

//TODO: Remove the ignore attribute after framework fix is implemented
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
#[ignore = "This should fail but for now the failing logs are not retrieved by the framework"]
// #[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_execute_flow_with_transfer_data_only_fail(#[case] shard: u32) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee(shard).await;

    chain_interactor
        .execute_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(WRONG_ENDPOINT_NAME.to_string())
                .expect_error(FUNCTION_NOT_FOUND.to_string())
                .expect_log(vec!["".to_string()]),
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
#[case::fungible(EsdtTokenType::Fungible)]
#[case::non_fungible(EsdtTokenType::NonFungibleV2)]
#[case::semi_fungible(EsdtTokenType::SemiFungible)]
#[case::meta_fungible(EsdtTokenType::MetaFungible)]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT)]
#[case::dynamic_sft(EsdtTokenType::DynamicSFT)]
#[case::dynamic_meta(EsdtTokenType::DynamicMeta)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_with_fee(
    #[case] token_type: EsdtTokenType,
    #[values(SHARD_1, SHARD_2)] shard: u32,
    #[values(0, 1)] token_index: usize,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    let token = chain_interactor.get_token_by_type(token_type, token_index);

    let fee = chain_interactor.create_standard_fee();

    chain_interactor.set_fee(fee.clone(), shard).await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new().shard(shard).expect_log(vec![
                DEPOSIT_LOG.to_string(),
                token.clone().token_id.into_managed_buffer().to_string(),
            ]),
            Some(token),
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
#[case::fungible(EsdtTokenType::Fungible)]
#[case::non_fungible(EsdtTokenType::NonFungibleV2)]
#[case::semi_fungible(EsdtTokenType::SemiFungible)]
#[case::meta_fungible(EsdtTokenType::MetaFungible)]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT)]
#[case::dynamic_sft(EsdtTokenType::DynamicSFT)]
#[case::dynamic_meta(EsdtTokenType::DynamicMeta)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_without_fee_and_execute(
    #[case] token_type: EsdtTokenType,
    #[values(SHARD_1, SHARD_2)] shard: u32,
    #[values(0, 1)] token_index: usize,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    let token = chain_interactor.get_token_by_type(token_type, token_index);

    chain_interactor.remove_fee(shard).await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new().shard(shard).expect_log(vec![
                DEPOSIT_LOG.to_string(),
                token.clone().token_id.into_managed_buffer().to_string(),
            ]),
            Some(token.clone()),
            None,
        )
        .await;

    chain_interactor
        .execute_wrapper(
            ActionConfig::new().shard(shard).expect_log(vec![token
                .clone()
                .token_id
                .into_managed_buffer()
                .to_string()]),
            Some(token),
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
async fn test_register_execute_and_deposit_sov_token(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee(shard).await;

    let (nonce, decimals) = chain_interactor.generate_nonce_and_decimals(token_type);
    let token_id = chain_interactor.create_random_sovereign_token_id(shard);

    let sov_token = EsdtTokenInfo {
        token_id: EgldOrEsdtTokenIdentifier::from(token_id.as_str()),
        nonce,
        token_type,
        decimals,
        amount,
    };

    let main_token = chain_interactor
        .register_and_execute_sovereign_token(ActionConfig::new().shard(shard), sov_token.clone())
        .await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new().shard(shard).expect_log(vec![
                sov_token.clone().token_id.into_managed_buffer().to_string(),
                main_token
                    .clone()
                    .token_id
                    .into_managed_buffer()
                    .to_string(),
            ]),
            Some(main_token),
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
#[case::fungible(EsdtTokenType::Fungible)]
#[case::non_fungible(EsdtTokenType::NonFungibleV2)]
#[case::semi_fungible(EsdtTokenType::SemiFungible)]
#[case::meta_fungible(EsdtTokenType::MetaFungible)]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT)]
#[case::dynamic_sft(EsdtTokenType::DynamicSFT)]
#[case::dynamic_meta(EsdtTokenType::DynamicMeta)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_mvx_token_with_transfer_data(
    #[case] token_type: EsdtTokenType,
    #[values(SHARD_1, SHARD_2)] shard: u32,
    #[values(0, 1)] token_index: usize,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee(shard).await;

    let token = chain_interactor.get_token_by_type(token_type, token_index);

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(vec![
                    DEPOSIT_LOG.to_string(),
                    token.clone().token_id.into_managed_buffer().to_string(),
                ]),
            Some(token),
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
#[case::fungible(EsdtTokenType::Fungible)]
#[case::non_fungible(EsdtTokenType::NonFungibleV2)]
#[case::semi_fungible(EsdtTokenType::SemiFungible)]
#[case::meta_fungible(EsdtTokenType::MetaFungible)]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT)]
#[case::dynamic_sft(EsdtTokenType::DynamicSFT)]
#[case::dynamic_meta(EsdtTokenType::DynamicMeta)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_mvx_token_with_transfer_data_and_fee(
    #[case] token_type: EsdtTokenType,
    #[values(SHARD_1, SHARD_2)] shard: u32,
    #[values(0, 1)] token_index: usize,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    let fee = chain_interactor.create_standard_fee();

    chain_interactor.set_fee(fee.clone(), shard).await;

    let token = chain_interactor.get_token_by_type(token_type, token_index);

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(vec![
                    DEPOSIT_LOG.to_string(),
                    token.clone().token_id.into_managed_buffer().to_string(),
                ]),
            Some(token),
            Some(fee),
        )
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call deposit without fee and execute operation with transfer data for various ESDT tokens.
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract and the event is found in logs for all token types.
#[rstest]
#[case::fungible(EsdtTokenType::Fungible)]
#[case::non_fungible(EsdtTokenType::NonFungibleV2)]
#[case::semi_fungible(EsdtTokenType::SemiFungible)]
#[case::meta_fungible(EsdtTokenType::MetaFungible)]
#[case::dynamic_nft(EsdtTokenType::DynamicNFT)]
#[case::dynamic_sft(EsdtTokenType::DynamicSFT)]
#[case::dynamic_meta(EsdtTokenType::DynamicMeta)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_and_execute_with_transfer_data(
    #[case] token_type: EsdtTokenType,
    #[values(0, 1)] token_index: usize,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    let token = chain_interactor.get_token_by_type(token_type, token_index);

    chain_interactor.remove_fee(shard).await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new().shard(shard).expect_log(vec![
                DEPOSIT_LOG.to_string(),
                token.clone().token_id.into_managed_buffer().to_string(),
            ]),
            Some(token.clone()),
            None,
        )
        .await;

    chain_interactor
        .execute_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(vec![token
                    .clone()
                    .token_id
                    .into_managed_buffer()
                    .to_string()]),
            Some(token.clone()),
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
async fn test_register_execute_with_transfer_data_and_deposit_sov_token(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee(shard).await;

    let (nonce, decimals) = chain_interactor.generate_nonce_and_decimals(token_type);
    let token_id = chain_interactor.create_random_sovereign_token_id(shard);

    let sov_token = EsdtTokenInfo {
        token_id: EgldOrEsdtTokenIdentifier::from(token_id.as_str()),
        nonce,
        token_type,
        decimals,
        amount: amount.clone(),
    };

    let main_token = chain_interactor
        .register_and_execute_sovereign_token(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string()),
            sov_token.clone(),
        )
        .await;

    chain_interactor
        .withdraw_from_testing_sc(
            main_token.clone(),
            main_token.nonce,
            main_token.amount.clone(),
        )
        .await;

    chain_interactor
        .deposit_wrapper(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(TESTING_SC_ENDPOINT.to_string())
                .expect_log(vec![
                    sov_token.clone().token_id.into_managed_buffer().to_string(),
                    main_token
                        .clone()
                        .token_id
                        .into_managed_buffer()
                        .to_string(),
                ]),
            Some(main_token.clone()),
            None,
        )
        .await;
}

//TODO: Remove the ignore attribute after framework fix is implemented
/// ### TEST
/// S-FORGE_COMPLETE-REGISTER_EXECUTE-FLOW_FAIL
///
/// ### ACTION
/// Deploy and complete setup phase, then call register, execute with transfer data
///
/// ### EXPECTED
/// The operation is not executed in the testing smart contract and the failed event is found in logs
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
#[ignore = "This should fail but for now the failing logs are not retrieved by the framework"]
// #[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_execute_call_failed(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
    #[values(SHARD_1, SHARD_2)] shard: u32,
) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee(shard).await;

    let (nonce, decimals) = chain_interactor.generate_nonce_and_decimals(token_type);
    let token_id = chain_interactor.create_random_sovereign_token_id(shard);

    let sov_token = EsdtTokenInfo {
        token_id: EgldOrEsdtTokenIdentifier::from(token_id.as_str()),
        nonce,
        token_type,
        decimals,
        amount,
    };

    chain_interactor
        .register_and_execute_sovereign_token(
            ActionConfig::new()
                .shard(shard)
                .with_endpoint(WRONG_ENDPOINT_NAME.to_string())
                .expect_error(FUNCTION_NOT_FOUND.to_string())
                .expect_log(vec!["".to_string()]),
            sov_token,
        )
        .await;
}

#[rstest]
#[case::async_call(SHARD_1)]
#[case::sync_call(SHARD_0)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_transfer_data_only_async_call_in_endpoint(#[case] shard: u32) {
    let mut chain_interactor = CompleteFlowInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee(shard).await;

    let mvx_esdt_safe_address = chain_interactor
        .common_state
        .get_mvx_esdt_safe_address(shard)
        .clone();

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from(READ_NATIVE_TOKEN_TESTING_SC_ENDPOINT);
    let args = ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![
        ManagedBuffer::new_from_bytes(mvx_esdt_safe_address.to_address().as_bytes()),
    ]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data = OperationData::new(
        chain_interactor
            .common_state()
            .get_and_increment_operation_nonce(&mvx_esdt_safe_address.to_string()),
        ManagedAddress::from_address(&chain_interactor.user_address),
        Some(transfer_data),
    );

    let operation = Operation::new(
        ManagedAddress::from_address(
            &chain_interactor
                .common_state()
                .current_testing_sc_address()
                .to_address(),
        ),
        ManagedVec::new(),
        operation_data,
    );

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor
        .register_operation(shard, &hash_of_hashes, operations_hashes)
        .await;

    let expected_operation_hash_status = OperationHashStatus::NotLocked;
    chain_interactor
        .check_registered_operation_status(
            shard,
            &hash_of_hashes,
            operation_hash,
            expected_operation_hash_status,
        )
        .await;

    let bridge_service = chain_interactor.get_bridge_service_for_shard(shard).clone();
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            bridge_service,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(""),
            None,
        )
        .await;
}
