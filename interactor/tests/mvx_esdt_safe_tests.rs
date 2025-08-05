use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_interactor::interactor_helpers::InteractorHelpers;
use common_interactor::interactor_structs::BalanceCheckConfig;
use common_test_setup::base_setup::init::RegisterTokenArgs;
use common_test_setup::constants::{
    CROWD_TOKEN_ID, DEPLOY_COST, DEPOSIT_LOG, EXECUTED_BRIDGE_LOG, FIRST_TEST_TOKEN, GAS_LIMIT,
    ISSUE_COST, MVX_TO_SOV_TOKEN_STORAGE_KEY, NATIVE_TOKEN_STORAGE_KEY, ONE_HUNDRED_TOKENS,
    ONE_THOUSAND_TOKENS, OPERATION_HASH_STATUS_STORAGE_KEY, PER_GAS, PER_TRANSFER,
    PREFERRED_CHAIN_IDS, SC_CALL_LOG, SHARD_0, SOVEREIGN_RECEIVER_ADDRESS, SOV_TOKEN,
    SOV_TO_MVX_TOKEN_STORAGE_KEY, TEN_TOKENS, TESTING_SC_ENDPOINT, TOKEN_TICKER,
    WRONG_ENDPOINT_NAME,
};
use cross_chain::MAX_GAS_PER_TRANSACTION;
use error_messages::{
    BANNED_ENDPOINT_NAME, CANNOT_REGISTER_TOKEN, CURRENT_OPERATION_NOT_REGISTERED,
    DEPOSIT_OVER_MAX_AMOUNT, ERR_EMPTY_PAYMENTS, GAS_LIMIT_TOO_HIGH, INVALID_TYPE,
    MAX_GAS_LIMIT_PER_TX_EXCEEDED, NATIVE_TOKEN_ALREADY_REGISTERED, NOTHING_TO_TRANSFER,
    PAYMENT_DOES_NOT_COVER_FEE, TOO_MANY_TOKENS,
};
use header_verifier::OperationHashStatus;
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_snippets::{hex, imports::*};
use rust_interact::mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use serial_test::serial;
use std::vec;
use structs::aliases::PaymentsVec;
use structs::configs::{EsdtSafeConfig, MaxBridgedAmount};
use structs::fee::{FeeStruct, FeeType};
use structs::forge::ScArray;
use structs::generate_hash::GenerateHash;
use structs::operation::{Operation, OperationData, OperationEsdtPayment, TransferData};

/// ### TEST
/// M-ESDT_DEPLOY_FAIL
///
/// ### ACTION
/// Call 'update_configuration()' with invalid config
///
/// ### EXPECTED
/// Error 'failedBridgeOp' log
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_update_invalid_config() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        MAX_GAS_PER_TRANSACTION + 1,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    let config_hash = config.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&config_hash.to_vec()));

    chain_interactor
        .register_operation(
            shard,
            ManagedBuffer::new(),
            &hash_of_hashes,
            MultiValueEncoded::from_iter(vec![config_hash]),
        )
        .await;

    chain_interactor
        .update_configuration(
            shard,
            hash_of_hashes,
            config,
            None,
            Some("failedBridgeOp"),
            Some(MAX_GAS_LIMIT_PER_TX_EXCEEDED),
        )
        .await;
}

/// ### TEST
/// M-ESDT_REG_FAIL
///
/// ### ACTION
/// Call 'register_token()' with invalid token type
///
/// ### EXPECTED
/// Error CANNOT_REGISTER_TOKEN
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_token_invalid_type_token_no_prefix() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let sov_token_id = TokenIdentifier::from_esdt_bytes(FIRST_TEST_TOKEN.as_str());
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "SOVEREIGN";
    let num_decimals = 18;
    let token_ticker = TOKEN_TICKER;
    let egld_payment = BigUint::from(ISSUE_COST);

    chain_interactor
        .register_token(
            SHARD_0,
            RegisterTokenArgs {
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            egld_payment,
            Some(CANNOT_REGISTER_TOKEN),
        )
        .await;

    let key = hex::encode(MVX_TO_SOV_TOKEN_STORAGE_KEY);
    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            key.as_str(),
            None,
        )
        .await;
}

/// ### TEST
/// M-ESDT_REG_FAIL
///
/// ### ACTION
/// Call 'register_token()' with invalid token type
///
/// ### EXPECTED
/// Error CANNOT_REGISTER_TOKEN
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_token_invalid_type_token_with_prefix() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let sov_token_id = TokenIdentifier::from_esdt_bytes(SOV_TOKEN.as_str());
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "SOVEREIGN";
    let num_decimals = 18;
    let token_ticker = TOKEN_TICKER;
    let egld_payment = BigUint::from(ISSUE_COST);

    chain_interactor
        .register_token(
            shard,
            RegisterTokenArgs {
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            egld_payment,
            Some(INVALID_TYPE),
        )
        .await;

    let key = hex::encode(MVX_TO_SOV_TOKEN_STORAGE_KEY);
    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            key.as_str(),
            None,
        )
        .await;
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_max_bridged_amount_exceeded() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from(TESTING_SC_ENDPOINT)]),
        ManagedVec::from(vec![MaxBridgedAmount {
            token_id: chain_interactor.state.get_first_token_identifier(),
            amount: BigUint::default(),
        }]),
    );

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::Some(config),
            None,
        )
        .await;

    let esdt_token_payment = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment]);

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::None,
            payments_vec,
            Some(DEPOSIT_OVER_MAX_AMOUNT),
            None,
        )
        .await;
    println!("Shard: {}", shard);
    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_all_contracts_empty(shard).await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with empty payments_vec and no transfer_data
///
/// ### EXPECTED
/// Error NOTHING_TO_TRANSFER
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_nothing_to_transfer() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::None,
            ManagedVec::new(),
            Some(NOTHING_TO_TRANSFER),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_all_contracts_empty(shard).await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with too many tokens in payments_vec
///
/// ### EXPECTED
/// Error TOO_MANY_TOKENS
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_too_many_tokens_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let esdt_token_payment = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        BigUint::from(1u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment; 11]);

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::None,
            payments_vec,
            Some(TOO_MANY_TOKENS),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_all_contracts_empty(shard).await;
}

/// ### TEST
/// M-ESDT_DEP_OK
///
/// ### ACTION
/// Call 'deposit()' with no transfer_data
///
/// ### EXPECTED
/// The deposit is successful
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_no_transfer_data() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one]);

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::None,
            payments_vec,
            None,
            Some(DEPOSIT_LOG),
        )
        .await;

    let first_token_id = chain_interactor.state.get_first_token_id();
    let expected_mvx_balance =
        chain_interactor.custom_amount_tokens(first_token_id.clone(), ONE_HUNDRED_TOKENS.into());

    chain_interactor
        .check_mvx_balance(shard, vec![expected_mvx_balance])
        .await;

    let balance_config = BalanceCheckConfig::new()
        .shard(shard)
        .token(Some(first_token_id))
        .amount(Some(ONE_HUNDRED_TOKENS.into()));

    chain_interactor
        .check_balances_after_action(balance_config)
        .await;

    chain_interactor
        .check_fee_market_balance(shard, Vec::new())
        .await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with gas limit too high in transfer_data
///
/// ### EXPECTED
/// Error GAS_LIMIT_TOO_HIGH
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_gas_limit_too_high_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        1,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::Some(config),
            None,
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one]);

    let gas_limit = 2u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some(GAS_LIMIT_TOO_HIGH),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_all_contracts_empty(shard).await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with banned endpoint name in transfer_data
///
/// ### EXPECTED
/// Error BANNED_ENDPOINT_NAME
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_endpoint_banned_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from(TESTING_SC_ENDPOINT)]),
        ManagedVec::new(),
    );

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::Some(config),
            None,
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one]);

    let gas_limit = 2u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some(BANNED_ENDPOINT_NAME),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_all_contracts_empty(shard).await;
}

/// ### TEST
/// M-ESDT_DEP_OK
///
/// ### ACTION
/// Call 'deposit()' with transfer data and valid payment
///
/// ### EXPECTED
/// USER's balance is updated
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_fee_enabled() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    let per_transfer = BigUint::from(PER_TRANSFER);
    let per_gas = BigUint::from(PER_GAS);

    let fee = FeeStruct {
        base_token: chain_interactor.state.get_fee_token_identifier(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_fee_token_identifier(),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            Some(fee.clone()),
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let fee_amount = BigUint::from(PER_TRANSFER) + (BigUint::from(GAS_LIMIT) * per_gas);

    let fee_payment = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_fee_token_identifier(),
        0,
        fee_amount.clone(),
    );

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![fee_payment, esdt_token_payment_one]);

    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((GAS_LIMIT, function, args));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::Some(transfer_data),
            payments_vec.clone(),
            None,
            Some(
                &chain_interactor
                    .state
                    .get_first_token_identifier()
                    .to_string(),
            ),
        )
        .await;

    let first_token = chain_interactor.state.get_first_token_id();

    println!("Fee amount: {}", fee_amount.to_display());

    let balance_config = BalanceCheckConfig::new()
        .shard(shard)
        .token(Some(first_token.clone()))
        .amount(Some(ONE_HUNDRED_TOKENS.into()))
        .fee(Some(fee))
        .with_transfer_data(true);

    chain_interactor
        .check_balances_after_action(balance_config)
        .await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with transfer data and no payment
///
/// ### EXPECTED
/// Error ERR_EMPTY_PAYMENTS
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_transfer_data_only_with_fee_nothing_to_transfer() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    let per_transfer = BigUint::from(1u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: chain_interactor.state.get_fee_token_identifier(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_fee_token_identifier(),
            per_transfer: per_transfer.clone(),
            per_gas,
        },
    };

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::Some(config),
            Some(fee),
        )
        .await;

    let gas_limit = 1000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::Some(transfer_data),
            ManagedVec::new(),
            Some(ERR_EMPTY_PAYMENTS),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_all_contracts_empty(shard).await;
}

/// ### TEST
/// M-ESDT_DEP_OK
///
/// ### ACTION
/// Call 'deposit()' with transfer data only and no payments
///
/// ### EXPECTED
/// The endpoint is called in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_only_transfer_data_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::Some(config),
            None,
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let gas_limit = 1000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::Some(transfer_data),
            ManagedVec::new(),
            None,
            Some(SC_CALL_LOG),
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_all_contracts_empty(shard).await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with transfer data and payment not enough for fee
///
/// ### EXPECTED
/// Error PAYMENT_DOES_NOT_COVER_FEE
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_payment_does_not_cover_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    let per_transfer = BigUint::from(1u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: chain_interactor.state.get_fee_token_identifier(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_fee_token_identifier(),
            per_transfer,
            per_gas,
        },
    };

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::Some(config),
            Some(fee),
        )
        .await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let fee_payment = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_fee_token_identifier(),
        0,
        BigUint::from(1u64),
    );

    let payments_vec = PaymentsVec::from(vec![fee_payment, esdt_token_payment_one]);

    let gas_limit = 10_000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some(PAYMENT_DOES_NOT_COVER_FEE),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_all_contracts_empty(shard).await;
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_refund() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;
    let user_address = chain_interactor.user_address().clone();

    let config = EsdtSafeConfig::new(
        ManagedVec::from(vec![TokenIdentifier::from(CROWD_TOKEN_ID)]),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: chain_interactor.state.get_fee_token_identifier(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_fee_token_identifier(),
            per_transfer,
            per_gas,
        },
    };

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::Some(config),
            Some(fee),
        )
        .await;

    let fee_amount = BigUint::from(ONE_THOUSAND_TOKENS);

    let fee_payment = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_fee_token_identifier(),
        0,
        fee_amount.clone(),
    );

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        BigUint::from(ONE_THOUSAND_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![fee_payment, esdt_token_payment_one]);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::Some(transfer_data),
            payments_vec.clone(),
            None,
            Some(
                &chain_interactor
                    .state
                    .get_first_token_identifier()
                    .to_string(),
            ),
        )
        .await;

    let expected_tokens_wallet = vec![chain_interactor.custom_amount_tokens(
        chain_interactor.state.get_fee_token_id(),
        (ONE_THOUSAND_TOKENS - gas_limit as u128).into(),
    )];
    chain_interactor
        .check_address_balance(&Bech32Address::from(user_address), expected_tokens_wallet)
        .await;

    chain_interactor.check_mvx_balance(shard, Vec::new()).await;

    let expected_fee_market_balance = chain_interactor
        .custom_amount_tokens(chain_interactor.state.get_fee_token_id(), gas_limit.into());
    chain_interactor
        .check_fee_market_balance(shard, vec![expected_fee_market_balance.clone()])
        .await;

    chain_interactor.check_testing_sc_balance(Vec::new()).await;
}

/// ### TEST
/// M-ESDT_REG_OK
///
/// ### ACTION
/// Call 'register_native_token()' with valid token id and name
///
/// ### EXPECTED
/// The token is registered
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_native_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor.get_bridge_owner_for_shard(shard).clone(),
            PREFERRED_CHAIN_IDS[0].to_string(),
            OptionalValue::None,
        )
        .await;

    let token_display_name = "SOVEREIGN";
    let token_ticker = TOKEN_TICKER;
    let egld_payment = BigUint::from(ISSUE_COST);

    chain_interactor
        .register_native_token(token_ticker, token_display_name, egld_payment, None)
        .await;

    let encoded_token_ticker = hex::encode(token_ticker);
    let encoded_key = &hex::encode(NATIVE_TOKEN_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            encoded_key,
            Some(&encoded_token_ticker),
        )
        .await;
}

/// ### TEST
/// M-ESDT_REG_OK
///
/// ### ACTION
/// Call 'register_native_token()' with valid token id and name
///
/// ### EXPECTED
/// The token is registered
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_native_token_twice() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor.get_bridge_owner_for_shard(shard).clone(),
            PREFERRED_CHAIN_IDS[0].to_string(),
            OptionalValue::None,
        )
        .await;

    let token_display_name = "SOVEREIGN";
    let token_ticker = TOKEN_TICKER;
    let egld_payment = BigUint::from(ISSUE_COST);

    chain_interactor
        .register_native_token(token_ticker, token_display_name, egld_payment.clone(), None)
        .await;

    let encoded_token_ticker = hex::encode(token_ticker);
    let encoded_key = &hex::encode(NATIVE_TOKEN_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            encoded_key,
            Some(&encoded_token_ticker),
        )
        .await;

    chain_interactor
        .register_native_token(
            token_ticker,
            token_display_name,
            egld_payment,
            Some(NATIVE_TOKEN_ALREADY_REGISTERED),
        )
        .await;
}

/// ### TEST
/// M-ESDT_REG_OK
///
/// ### ACTION
/// Call 'register_token()' with valid token id and type
///
/// ### EXPECTED
/// The token is registered
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_token_fungible_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_chain_config(
            chain_interactor.get_bridge_owner_for_shard(shard).clone(),
            PREFERRED_CHAIN_IDS[0].to_string(),
            OptionalValue::None,
        )
        .await;

    let contracts_array =
        chain_interactor.get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    chain_interactor
        .deploy_header_verifier(
            chain_interactor.get_bridge_owner_for_shard(shard).clone(),
            PREFERRED_CHAIN_IDS[0].to_string(),
            contracts_array,
        )
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor.get_bridge_owner_for_shard(shard).clone(),
            PREFERRED_CHAIN_IDS[0].to_string(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    let sov_token_id = TokenIdentifier::from_esdt_bytes(SOV_TOKEN.as_str());
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "GREEN";
    let num_decimals = 18;
    let token_ticker = TOKEN_TICKER;
    let egld_payment = BigUint::from(ISSUE_COST);

    chain_interactor
        .register_token(
            SHARD_0,
            RegisterTokenArgs {
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            egld_payment,
            None,
        )
        .await;

    let encoded_token_ticker = hex::encode(token_ticker);
    let encoded_key = &hex::encode(SOV_TO_MVX_TOKEN_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            encoded_key,
            Some(&encoded_token_ticker),
        )
        .await;
}

/// ### TEST
/// M-ESDT_REG_OK
///
/// ### ACTION
/// Call 'register_token()' with valid token id and non-fungible type
///
/// ### EXPECTED
/// The token is registered
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_token_non_fungible_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_chain_config(
            chain_interactor.get_bridge_owner_for_shard(shard).clone(),
            PREFERRED_CHAIN_IDS[0].to_string(),
            OptionalValue::None,
        )
        .await;

    let contracts_array =
        chain_interactor.get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    chain_interactor
        .deploy_header_verifier(
            chain_interactor.get_bridge_owner_for_shard(shard).clone(),
            PREFERRED_CHAIN_IDS[0].to_string(),
            contracts_array,
        )
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor.get_bridge_owner_for_shard(shard).clone(),
            PREFERRED_CHAIN_IDS[0].to_string(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    let sov_token_id = TokenIdentifier::from_esdt_bytes(SOV_TOKEN.as_str());
    let token_type = EsdtTokenType::NonFungible;
    let token_display_name = "SOVEREIGN";
    let num_decimals = 18;
    let token_ticker = TOKEN_TICKER;
    let egld_payment = BigUint::from(ISSUE_COST);

    chain_interactor
        .register_token(
            SHARD_0,
            RegisterTokenArgs {
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            egld_payment,
            None,
        )
        .await;

    let encoded_token_ticker = hex::encode(token_ticker);
    let encoded_key = &hex::encode(SOV_TO_MVX_TOKEN_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            encoded_key,
            Some(&encoded_token_ticker),
        )
        .await;
}

/// ### TEST
/// M-ESDT_REG_OK
///
/// ### ACTION
/// Call 'register_token()' with valid token id and dynamic NFT type
///
/// ### EXPECTED
/// The token is registered
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_token_dynamic_non_fungible_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_chain_config(
            chain_interactor.get_bridge_owner_for_shard(shard).clone(),
            PREFERRED_CHAIN_IDS[0].to_string(),
            OptionalValue::None,
        )
        .await;

    let contracts_array =
        chain_interactor.get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    chain_interactor
        .deploy_header_verifier(
            chain_interactor.get_bridge_owner_for_shard(shard).clone(),
            PREFERRED_CHAIN_IDS[0].to_string(),
            contracts_array,
        )
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor.get_bridge_owner_for_shard(shard).clone(),
            PREFERRED_CHAIN_IDS[0].to_string(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    let sov_token_id = TokenIdentifier::from_esdt_bytes(SOV_TOKEN.as_str());
    let token_type = EsdtTokenType::DynamicNFT;
    let token_display_name = "SOVEREIGN";
    let num_decimals = 18;
    let token_ticker = TOKEN_TICKER;
    let egld_payment = BigUint::from(ISSUE_COST);

    chain_interactor
        .register_token(
            SHARD_0,
            RegisterTokenArgs {
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            egld_payment,
            None,
        )
        .await;

    let encoded_token_ticker = hex::encode(token_ticker);
    let encoded_key = &hex::encode(SOV_TO_MVX_TOKEN_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            encoded_key,
            Some(&encoded_token_ticker),
        )
        .await;
}

/// ### TEST
/// M-ESDT_EXEC_FAIL
///
/// ### ACTION
/// Call 'execute_operation()' with no esdt-safe-address set
///
/// ### EXPECTED
/// Error NO_ESDT_SAFE_ADDRESS
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_no_operation_registered() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let payment = OperationEsdtPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        EsdtTokenData::default(),
    );

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        None,
    );

    let operation = Operation::new(
        ManagedAddress::from_address(
            &chain_interactor
                .state
                .current_testing_sc_address()
                .to_address(),
        ),
        vec![payment].into(),
        operation_data,
    );

    let hash_of_hashes = chain_interactor.get_operation_hash(&operation);

    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            chain_interactor.get_bridge_service_for_shard(shard).clone(),
            shard,
            hash_of_hashes,
            operation,
            Some(CURRENT_OPERATION_NOT_REGISTERED),
            None,
            None,
        )
        .await;

    let encoded_key = &hex::encode(OPERATION_HASH_STATUS_STORAGE_KEY);
    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_all_contracts_empty(shard).await;
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_success_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;
    let token_data = EsdtTokenData {
        amount: BigUint::from(TEN_TOKENS),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        token_data,
    );
    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: chain_interactor.state.get_first_token_identifier(),
        token_nonce: 0,
        amount: BigUint::from(TEN_TOKENS),
    });

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        Some(transfer_data),
    );

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let operation = Operation::new(
        ManagedAddress::from_address(
            &chain_interactor
                .state
                .current_testing_sc_address()
                .to_address(),
        ),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            shard,
            OptionalValue::None,
            payment_vec,
            None,
            Some(DEPOSIT_LOG),
        )
        .await;

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor
        .register_operation(
            shard,
            ManagedBuffer::new(),
            &hash_of_hashes,
            operations_hashes,
        )
        .await;

    let operation_status = OperationHashStatus::NotLocked as u8;
    let expected_operation_hash_status = format!("{:02x}", operation_status);
    let encoded_key = &hex::encode(OPERATION_HASH_STATUS_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            chain_interactor.get_bridge_service_for_shard(shard).clone(),
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(EXECUTED_BRIDGE_LOG),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let balance_config = BalanceCheckConfig::new()
        .shard(shard)
        .token(Some(chain_interactor.state.get_first_token_id()))
        .amount(Some(TEN_TOKENS.into()))
        .is_execute(true)
        .with_transfer_data(true);

    chain_interactor
        .check_balances_after_action(balance_config)
        .await;

    chain_interactor.check_mvx_balance(shard, Vec::new()).await;

    chain_interactor
        .check_fee_market_balance(shard, Vec::new())
        .await;

    let expected_testing_sc_balance = vec![chain_interactor.custom_amount_tokens(
        chain_interactor.state.get_first_token_id(),
        TEN_TOKENS.into(),
    )];
    chain_interactor
        .check_testing_sc_balance(expected_testing_sc_balance)
        .await;
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation and no fee
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_only_transfer_data_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        Some(transfer_data),
    );

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let operation = Operation::new(
        ManagedAddress::from_address(
            &chain_interactor
                .state
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
        .register_operation(
            shard,
            ManagedBuffer::new(),
            &hash_of_hashes,
            operations_hashes,
        )
        .await;

    let operation_status = OperationHashStatus::NotLocked as u8;
    let expected_operation_hash_status = format!("{:02x}", operation_status);
    let encoded_key = &hex::encode(OPERATION_HASH_STATUS_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            chain_interactor.get_bridge_service_for_shard(shard).clone(),
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(EXECUTED_BRIDGE_LOG),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_all_contracts_empty(shard).await;
}

/// ### TEST
/// M-ESDT_EXEC_FAIL
///
/// ### ACTION
/// Call 'execute_operation()' with invalid endpoint in transfer data
///
/// ### EXPECTED
/// The testing smart contract returns a failed event
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_no_payments_failed_event() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from(WRONG_ENDPOINT_NAME);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function.clone(), args);

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        Some(transfer_data),
    );

    chain_interactor
        .deploy_and_complete_setup_phase_on_a_shard(
            shard,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let operation = Operation::new(
        ManagedAddress::from_address(
            &chain_interactor
                .state
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
        .register_operation(
            shard,
            ManagedBuffer::new(),
            &hash_of_hashes,
            operations_hashes,
        )
        .await;

    let operation_status = OperationHashStatus::NotLocked as u8;
    let expected_operation_hash_status = format!("{:02x}", operation_status);
    let encoded_key = &hex::encode(OPERATION_HASH_STATUS_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            chain_interactor.get_bridge_service_for_shard(shard).clone(),
            shard,
            hash_of_hashes,
            operation,
            Some(function.to_string().as_str()),
            None,
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_all_contracts_empty(shard).await;
}
