use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_interactor::interactor_state::EsdtTokenInfo;
use common_interactor::interactor_state::TokenBalance;
use common_test_setup::base_setup::init::RegisterTokenArgs;
use common_test_setup::constants::GAS_LIMIT;
use common_test_setup::constants::ONE_THOUSAND_TOKENS;
use common_test_setup::constants::PER_GAS;
use common_test_setup::constants::PER_TRANSFER;
use common_test_setup::constants::{
    DEPLOY_COST, ISSUE_COST, ONE_HUNDRED_TOKENS, REGISTER_TOKEN_PREFIX, SC_CALL_LOG, SHARD_0,
    SHARD_1, SHARD_2, TESTING_SC_ENDPOINT, TOKEN_DISPLAY_NAME, TOKEN_TICKER, WRONG_ENDPOINT_NAME,
};
use multiversx_sc::imports::Bech32Address;
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
/// Deploy and complete setup phase, then call deposit_in_mvx_esdt_safe
///
/// ### EXPECTED
/// Deposit is successful and the event is found in logs
#[rstest]
#[case(SHARD_2)]
#[case(SHARD_1)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_deposit_flow_no_fee_only_transfer_data(#[case] shard: u32) {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .complete_deposit_flow_with_transfer_data_only(shard, None, Some(SC_CALL_LOG))
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call deposit_in_mvx_esdt_safe
///
/// ### EXPECTED
/// Deposit is successful and the event is found in logs
#[rstest]
#[case(SHARD_2)]
#[case(SHARD_1)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_deposit_flow_with_fee_only_transfer_data(#[case] shard: u32) {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    let fee = chain_interactor.create_standard_fee();

    chain_interactor
        .complete_deposit_flow_with_transfer_data_only(shard, Some(fee), Some(SC_CALL_LOG))
        .await;
}

/// ### TEST
/// S-FORGE_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[rstest]
#[case(SHARD_2)]
#[case(SHARD_1)]
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
}

/// ### TEST
/// S-FORGE_EXEC_FAIL
///
/// ### ACTION
/// Call 'execute_operation()' with invalid operation
///
/// ### EXPECTED
/// The operation is not executed in the testing smart contract
#[rstest]
#[case(SHARD_2)]
#[case(SHARD_1)]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_execute_flow_with_transfer_data_only_fail_different_shard(
    #[case] shard: u32,
) {
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
}

#[rstest]
#[case(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_with_fee(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
) {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

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
        .deposit_no_transfer_data(shard, token.clone(), amount.clone(), Some(fee))
        .await;

    let expected_changed_user_balances = vec![
        TokenBalance {
            token_id: token.token_id.clone(),
            amount: chain_interactor
                .state
                .get_initial_token_balance_for_address(
                    chain_interactor.user_address.clone().into(),
                    TokenIdentifier::from(&token.token_id),
                )
                - amount.clone(),
        },
        TokenBalance {
            token_id: chain_interactor.state.get_fee_token_id_string(),
            amount: BigUint::from(ONE_THOUSAND_TOKENS) - PER_TRANSFER,
        },
    ];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_changed_user_balances,
        )
        .await;

    let expected_mvx_balance = vec![TokenBalance {
        token_id: token.token_id.clone(),
        amount,
    }];
    let mvx_esdt_safe_address = chain_interactor
        .state
        .get_mvx_esdt_safe_address(shard)
        .clone();
    chain_interactor
        .check_address_balance(&mvx_esdt_safe_address, expected_mvx_balance)
        .await;
}

#[rstest]
#[case(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_without_fee_and_execute(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
) {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_2;

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

    let expected_changed_user_balances = vec![TokenBalance {
        token_id: token.token_id.clone(),
        amount: chain_interactor
            .state
            .get_initial_token_balance_for_address(
                chain_interactor.user_address().clone().into(),
                TokenIdentifier::from(&token.token_id),
            )
            - amount.clone(),
    }];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_changed_user_balances,
        )
        .await;

    let expected_mvx_balance = vec![TokenBalance {
        token_id: token.token_id.clone(),
        amount: amount.clone(),
    }];
    let mvx_esdt_safe_address = chain_interactor
        .state
        .get_mvx_esdt_safe_address(shard)
        .clone();
    chain_interactor
        .check_address_balance(&mvx_esdt_safe_address, expected_mvx_balance)
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

    let expected_changed_user_balances = vec![TokenBalance {
        token_id: token.token_id.clone(),
        amount: BigUint::from(ONE_THOUSAND_TOKENS),
    }];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_changed_user_balances,
        )
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
}

#[rstest]
#[case(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_execute_and_deposit_sov_token(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
) {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let token = chain_interactor.get_token_by_type(token_type);

    let sov_token_id = TokenIdentifier::from_esdt_bytes(
        REGISTER_TOKEN_PREFIX.to_string() + &token.token_id.clone(),
    );
    let token_display_name = TOKEN_DISPLAY_NAME;
    let num_decimals = 18;
    let wanted_token_id = token.token_id.clone();
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
        nonce: token.nonce,
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

    let expected_changed_user_balances = vec![TokenBalance {
        token_id: expected_token.to_string(),
        amount: amount.clone(),
    }];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_changed_user_balances,
        )
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;

    let mut nonce = token.nonce;
    if token_type == EsdtTokenType::NonFungibleV2 || token_type == EsdtTokenType::DynamicNFT {
        let sov_token_info = chain_interactor
            .get_sov_to_mvx_token_id_with_nonce(shard, sov_token_id, token.nonce)
            .await;
        nonce = sov_token_info.token_nonce;
    }

    let deposit_token_info = EsdtTokenInfo {
        token_id: expected_token.to_string(),
        nonce,
    };

    chain_interactor
        .deposit_no_transfer_data(shard, deposit_token_info, amount, None)
        .await;

    chain_interactor
        .check_initial_wallet_balance_unchanged()
        .await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
}

#[rstest]
#[case(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_mvx_token_with_transfer_data(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
) {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_2;

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

    let expected_changed_wallet_balances = vec![TokenBalance {
        token_id: token.token_id.clone(),
        amount: chain_interactor
            .state
            .get_initial_token_balance_for_address(
                chain_interactor.user_address.clone().into(),
                TokenIdentifier::from(&token.token_id),
            )
            - amount.clone(),
    }];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_changed_wallet_balances,
        )
        .await;

    let expected_mvx_balance = vec![TokenBalance {
        token_id: token.token_id.clone(),
        amount: amount.clone(),
    }];
    let mvx_esdt_safe_address = chain_interactor
        .state
        .get_mvx_esdt_safe_address(shard)
        .clone();
    chain_interactor
        .check_address_balance(&mvx_esdt_safe_address, expected_mvx_balance)
        .await;
}

#[rstest]
#[case(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_mvx_token_with_transfer_data_and_fee(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
) {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

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
        .deposit_with_transfer_data(shard, token.clone(), amount.clone(), Some(fee))
        .await;

    let expected_changed_wallet_balances = vec![
        TokenBalance {
            token_id: token.token_id.clone(),
            amount: chain_interactor
                .state
                .get_initial_token_balance_for_address(
                    chain_interactor.user_address.clone().into(),
                    TokenIdentifier::from(&token.token_id),
                )
                - amount.clone(),
        },
        TokenBalance {
            token_id: chain_interactor.state.get_fee_token_id_string(),
            amount: BigUint::from(ONE_THOUSAND_TOKENS) - PER_TRANSFER - PER_GAS * GAS_LIMIT,
        },
    ];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_changed_wallet_balances,
        )
        .await;

    let expected_mvx_balance = vec![TokenBalance {
        token_id: token.token_id.clone(),
        amount: amount.clone(),
    }];
    let mvx_esdt_safe_address = chain_interactor
        .state
        .get_mvx_esdt_safe_address(shard)
        .clone();
    chain_interactor
        .check_address_balance(&mvx_esdt_safe_address, expected_mvx_balance)
        .await;
}

#[rstest]
#[case(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_and_execute_with_transfer_data(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
) {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_2;

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

    let expected_changed_user_balances = vec![TokenBalance {
        token_id: token.token_id.clone(),
        amount: chain_interactor
            .state
            .get_initial_token_balance_for_address(
                chain_interactor.user_address.clone().into(),
                TokenIdentifier::from(&token.token_id),
            )
            - amount.clone(),
    }];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_changed_user_balances,
        )
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;

    let expected_testing_sc_balance = vec![TokenBalance {
        token_id: token.token_id.clone(),
        amount,
    }];
    let testing_sc = chain_interactor.state.current_testing_sc_address().clone();
    chain_interactor
        .check_address_balance(&testing_sc, expected_testing_sc_balance)
        .await;
}

#[rstest]
#[case(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_registed_execute_with_transfer_data_and_deposit_sov_token(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
) {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let token = chain_interactor.get_token_by_type(token_type);

    let sov_token_id = TokenIdentifier::from_esdt_bytes(
        REGISTER_TOKEN_PREFIX.to_string() + &token.token_id.clone(),
    );
    let token_display_name = TOKEN_DISPLAY_NAME;
    let num_decimals = 18;
    let wanted_token_id = token.token_id.clone();
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
        nonce: token.nonce,
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

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;

    chain_interactor
        .check_address_balance(
            &chain_interactor.state.current_testing_sc_address().clone(),
            vec![TokenBalance {
                token_id: expected_token.to_string(),
                amount: amount.clone(),
            }],
        )
        .await;

    let mut nonce = token.nonce;
    if token_type == EsdtTokenType::NonFungibleV2 || token_type == EsdtTokenType::DynamicNFT {
        let sov_token_info = chain_interactor
            .get_sov_to_mvx_token_id_with_nonce(shard, sov_token_id, token.nonce)
            .await;
        nonce = sov_token_info.token_nonce;
    }

    chain_interactor
        .withdraw_from_testing_sc(expected_token.clone(), nonce, amount.clone())
        .await;

    let deposit_token_info = EsdtTokenInfo {
        token_id: expected_token.to_string(),
        nonce,
    };

    chain_interactor
        .deposit_no_transfer_data(shard, deposit_token_info, amount, None)
        .await;

    chain_interactor
        .check_initial_wallet_balance_unchanged()
        .await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
}

#[rstest]
#[case(EsdtTokenType::Fungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::NonFungibleV2, BigUint::from(1u64))]
#[case(EsdtTokenType::SemiFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::MetaFungible, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicNFT, BigUint::from(1u64))]
#[case(EsdtTokenType::DynamicSFT, BigUint::from(ONE_HUNDRED_TOKENS))]
#[case(EsdtTokenType::DynamicMeta, BigUint::from(ONE_HUNDRED_TOKENS))]
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_execute_call_failed(
    #[case] token_type: EsdtTokenType,
    #[case] amount: BigUint<StaticApi>,
) {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let token = chain_interactor.get_token_by_type(token_type);

    let sov_token_id = TokenIdentifier::from_esdt_bytes(
        REGISTER_TOKEN_PREFIX.to_string() + &token.token_id.clone(),
    );
    let token_display_name = TOKEN_DISPLAY_NAME;
    let num_decimals = 18;
    let wanted_token_id = token.token_id.clone();
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
        nonce: token.nonce,
    };

    chain_interactor
        .execute_operation(
            shard,
            Some(WRONG_ENDPOINT_NAME),
            None,
            token_info,
            amount.clone(),
            Some(WRONG_ENDPOINT_NAME),
        )
        .await;

    chain_interactor
        .check_initial_wallet_balance_unchanged()
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
}
