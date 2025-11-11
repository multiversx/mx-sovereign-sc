use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_interactor::interactor_helpers::InteractorHelpers;
use common_interactor::interactor_state::EsdtTokenInfo;
use common_interactor::interactor_structs::{ActionConfig, BalanceCheckConfig};
use common_test_setup::base_setup::init::ExpectedLogs;
use common_test_setup::constants::{
    EGLD_0_05, GAS_LIMIT, MULTI_ESDT_NFT_TRANSFER_EVENT, ONE_HUNDRED_TOKENS, PER_GAS, PER_TRANSFER,
    SHARD_0, SHARD_1, SOVEREIGN_RECEIVER_ADDRESS, TEN_TOKENS, TESTING_SC_ENDPOINT,
};
use common_test_setup::log;
use cross_chain::MAX_GAS_PER_TRANSACTION;
use error_messages::{
    BANNED_ENDPOINT_NAME, CURRENT_OPERATION_NOT_REGISTERED, DEPOSIT_OVER_MAX_AMOUNT,
    ERR_EMPTY_PAYMENTS, GAS_LIMIT_TOO_HIGH, MAX_GAS_LIMIT_PER_TX_EXCEEDED, NOTHING_TO_TRANSFER,
};
use multiversx_sc::api::ESDT_LOCAL_MINT_FUNC_NAME;
use multiversx_sc::chain_core::EGLD_000000_TOKEN_IDENTIFIER;
use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use rust_interact::mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use serial_test::serial;
use std::vec;
use structs::aliases::PaymentsVec;
use structs::configs::{EsdtSafeConfig, MaxBridgedAmount};
use structs::operation::{Operation, OperationData, OperationEsdtPayment, TransferData};
use structs::OperationHashStatus;

//NOTE: The chain sim environment can not handle storage reads from other shards

/// ### TEST
/// M-ESDT_UPDATE_CONFIG_FAIL
///
/// ### ACTION
/// Call 'update_configuration()' with invalid config
///
/// ### EXPECTED
/// Error 'MAX_GAS_LIMIT_PER_TX_EXCEEDED' log
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_update_invalid_config() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let max_tx_gas_limit = MAX_GAS_PER_TRANSACTION + 1;
    let config = EsdtSafeConfig {
        max_tx_gas_limit,
        ..EsdtSafeConfig::default_config()
    };

    chain_interactor
        .update_configuration_after_setup_phase(
            SHARD_1,
            config,
            Some(MAX_GAS_LIMIT_PER_TX_EXCEEDED),
        )
        .await;
}

/// ### TEST
/// M-ESDT_DEPOSIT_FAIL
///
/// ### ACTION
/// Call 'deposit()' with max bridged amount exceeded
///
/// ### EXPECTED
/// Error 'DEPOSIT_OVER_MAX_AMOUNT' log
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_max_bridged_amount_exceeded() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    let config = EsdtSafeConfig {
        max_bridged_token_amounts: ManagedVec::from(vec![MaxBridgedAmount {
            token_id: chain_interactor.state.get_first_fungible_token_identifier(),
            amount: BigUint::default(),
        }]),
        ..EsdtSafeConfig::default_config()
    };

    chain_interactor
        .update_configuration_after_setup_phase(SHARD_1, config, None)
        .await;

    let esdt_token_payment = EgldOrEsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_fungible_token_identifier(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment]);

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_1,
            OptionalValue::None,
            payments_vec,
            Some(DEPOSIT_OVER_MAX_AMOUNT),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_1).await;

    chain_interactor
        .update_configuration_after_setup_phase(SHARD_1, EsdtSafeConfig::default_config(), None)
        .await;
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

    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_1,
            OptionalValue::None,
            ManagedVec::new(),
            Some(NOTHING_TO_TRANSFER),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_1).await;
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

    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    let esdt_token_payment_one = EgldOrEsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_fungible_token_identifier(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one]);

    let expected_logs = chain_interactor.build_expected_deposit_log(
        ActionConfig::new().shard(SHARD_1),
        Some(chain_interactor.state.get_first_fungible_token_id()),
    );
    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_1,
            OptionalValue::None,
            payments_vec,
            None,
            Some(expected_logs),
        )
        .await;

    let first_token_id = chain_interactor.state.get_first_fungible_token_id();

    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_1)
        .token(Some(first_token_id))
        .amount(ONE_HUNDRED_TOKENS.into());

    chain_interactor
        .check_balances_after_action(balance_config)
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

    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    let shard = SHARD_1;
    let config = EsdtSafeConfig {
        max_tx_gas_limit: 1,
        ..EsdtSafeConfig::default_config()
    };

    chain_interactor
        .update_configuration_after_setup_phase(SHARD_1, config, None)
        .await;

    let esdt_token_payment_one = EgldOrEsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_fungible_token_identifier(),
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
    chain_interactor.check_contracts_empty(shard).await;

    chain_interactor
        .update_configuration_after_setup_phase(SHARD_1, EsdtSafeConfig::default_config(), None)
        .await;
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

    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    let config = EsdtSafeConfig {
        banned_endpoints: ManagedVec::from(vec![ManagedBuffer::from(TESTING_SC_ENDPOINT)]),
        ..EsdtSafeConfig::default_config()
    };

    chain_interactor
        .update_configuration_after_setup_phase(SHARD_1, config, None)
        .await;

    let esdt_token_payment_one = EgldOrEsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_fungible_token_identifier(),
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
            SHARD_1,
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some(BANNED_ENDPOINT_NAME),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_1).await;

    chain_interactor
        .update_configuration_after_setup_phase(SHARD_1, EsdtSafeConfig::default_config(), None)
        .await;
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

    let fee = chain_interactor.create_standard_fee();
    chain_interactor.set_fee_wrapper(fee.clone(), SHARD_1).await;

    let fee_amount = BigUint::from(PER_TRANSFER) + (BigUint::from(GAS_LIMIT) * PER_GAS);

    let fee_payment = EgldOrEsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_fee_token_identifier(),
        0,
        fee_amount.clone(),
    );

    let esdt_token_payment_one = EgldOrEsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_fungible_token_identifier(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![fee_payment, esdt_token_payment_one]);

    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((GAS_LIMIT, function, args));

    let expected_logs = chain_interactor.build_expected_deposit_log(
        ActionConfig::new().shard(SHARD_1),
        Some(chain_interactor.state.get_first_fungible_token_id()),
    );
    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_1,
            OptionalValue::Some(transfer_data),
            payments_vec.clone(),
            None,
            Some(expected_logs),
        )
        .await;

    let first_token_id = chain_interactor.state.get_first_fungible_token_id();
    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_1)
        .token(Some(first_token_id.clone()))
        .amount(ONE_HUNDRED_TOKENS.into())
        .fee(Some(fee.clone()))
        .with_transfer_data(true);

    chain_interactor
        .check_balances_after_action(balance_config)
        .await;
    chain_interactor
        .update_fee_market_balance_state(Some(fee), payments_vec, SHARD_1)
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

    let fee = chain_interactor.create_standard_fee();

    chain_interactor.set_fee_wrapper(fee, SHARD_1).await;

    let gas_limit = 1000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_1,
            OptionalValue::Some(transfer_data),
            ManagedVec::new(),
            Some(ERR_EMPTY_PAYMENTS),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_1).await;
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

    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    let gas_limit = 1000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    let expected_logs =
        chain_interactor.build_expected_deposit_log(ActionConfig::new().shard(SHARD_1), None);
    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_1,
            OptionalValue::Some(transfer_data),
            ManagedVec::new(),
            None,
            Some(expected_logs),
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_1).await;
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

    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    let payment = OperationEsdtPayment::new(
        chain_interactor.state.get_first_fungible_token_identifier(),
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
                .common_state()
                .current_testing_sc_address()
                .to_address(),
        ),
        vec![payment].into(),
        operation_data,
    );

    let hash_of_hashes = chain_interactor.get_operation_hash(&operation);
    let bridge_service = chain_interactor
        .get_bridge_service_for_shard(SHARD_1)
        .clone();

    let expected_logs = chain_interactor.build_expected_execute_log(
        ActionConfig::new()
            .shard(SHARD_1)
            .expected_log_error(CURRENT_OPERATION_NOT_REGISTERED),
        Some(chain_interactor.state.get_first_fungible_token_id()),
    );
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            bridge_service,
            SHARD_1,
            hash_of_hashes,
            operation,
            None,
            Some(expected_logs),
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_1).await;
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
async fn test_execute_operation_with_egld_success_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    let token = EsdtTokenInfo {
        token_id: EgldOrEsdtTokenIdentifier::egld(),
        amount: BigUint::from(EGLD_0_05),
        nonce: 0,
        decimals: 18,
        token_type: EsdtTokenType::Fungible,
    };

    let token_data = EsdtTokenData {
        amount: BigUint::from(EGLD_0_05),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, token_data);
    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EgldOrEsdtTokenPayment::new(
        EgldOrEsdtTokenIdentifier::egld(),
        0,
        BigUint::from(EGLD_0_05),
    ));

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);
    let mvx_esdt_safe_address = chain_interactor
        .common_state
        .get_mvx_esdt_safe_address(SHARD_1)
        .clone();

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
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    let expected_log =
        vec![log!(MULTI_ESDT_NFT_TRANSFER_EVENT, topics: [EGLD_000000_TOKEN_IDENTIFIER])];
    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_1,
            OptionalValue::None,
            payment_vec,
            None,
            Some(expected_log),
        )
        .await;

    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_1)
        .token(Some(token.clone()))
        .amount(EGLD_0_05.into());

    chain_interactor
        .check_balances_after_action(balance_config)
        .await;

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor
        .register_operation(SHARD_1, &hash_of_hashes, operations_hashes)
        .await;

    let expected_operation_hash_status = OperationHashStatus::NotLocked;
    chain_interactor
        .check_registered_operation_status(
            SHARD_1,
            &hash_of_hashes,
            operation_hash,
            expected_operation_hash_status,
        )
        .await;

    let bridge_service = chain_interactor
        .get_bridge_service_for_shard(SHARD_1)
        .clone();
    let expected_logs = chain_interactor.build_expected_execute_log(
        ActionConfig::new()
            .shard(SHARD_1)
            .with_endpoint(TESTING_SC_ENDPOINT.to_string()),
        Some(token.clone()),
    );
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            bridge_service,
            SHARD_1,
            hash_of_hashes,
            operation,
            None,
            Some(expected_logs),
        )
        .await;

    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_1)
        .token(Some(token))
        .amount(EGLD_0_05.into())
        .is_execute(true)
        .with_transfer_data(true);

    chain_interactor
        .check_balances_after_action(balance_config)
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

    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);
    let mvx_esdt_safe_address = chain_interactor
        .common_state
        .get_mvx_esdt_safe_address(SHARD_1)
        .clone();

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
        .register_operation(SHARD_1, &hash_of_hashes, operations_hashes)
        .await;

    let expected_operation_status = OperationHashStatus::NotLocked;
    chain_interactor
        .check_registered_operation_status(
            SHARD_1,
            &hash_of_hashes,
            operation_hash,
            expected_operation_status,
        )
        .await;

    let bridge_service = chain_interactor
        .get_bridge_service_for_shard(SHARD_1)
        .clone();
    let expected_logs =
        chain_interactor.build_expected_execute_log(ActionConfig::new().shard(SHARD_1), None);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            bridge_service,
            SHARD_1,
            hash_of_hashes,
            operation,
            None,
            Some(expected_logs),
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_1).await;
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
async fn test_execute_operation_native_token_success_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    let token_data = EsdtTokenData {
        amount: BigUint::from(TEN_TOKENS),
        ..Default::default()
    };

    let native_token = chain_interactor.get_native_token(SHARD_1).await;

    let payment = OperationEsdtPayment::new(native_token.clone(), 0, token_data);

    let mvx_esdt_safe_address = chain_interactor
        .common_state
        .get_mvx_esdt_safe_address(SHARD_1)
        .clone();

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

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
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor
        .register_operation(SHARD_1, &hash_of_hashes, operations_hashes)
        .await;

    let expected_operation_hash_status = OperationHashStatus::NotLocked;
    chain_interactor
        .check_registered_operation_status(
            SHARD_1,
            &hash_of_hashes,
            operation_hash,
            expected_operation_hash_status,
        )
        .await;

    let bridge_service = chain_interactor
        .get_bridge_service_for_shard(SHARD_1)
        .clone();

    let native_token_info = EsdtTokenInfo {
        token_id: native_token.clone(),
        amount: BigUint::from(TEN_TOKENS),
        nonce: 0,
        decimals: 18,
        token_type: EsdtTokenType::Fungible,
    };

    let native_token_id = native_token.clone().into_managed_buffer().to_string();

    let expected_logs = vec![log!(ESDT_LOCAL_MINT_FUNC_NAME, topics: [native_token_id])];
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            bridge_service,
            SHARD_1,
            hash_of_hashes,
            operation,
            None,
            Some(expected_logs),
        )
        .await;

    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_1)
        .token(Some(native_token_info))
        .amount(TEN_TOKENS.into())
        .is_execute(true)
        .with_transfer_data(true);

    chain_interactor
        .check_balances_after_action(balance_config)
        .await;
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation but no registered sovereign token
///
/// ### EXPECTED
/// Transaction is not successful and the logs contain the InternalVMError
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_sovereign_token_not_registered() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    let sov_token_id = chain_interactor.create_random_sovereign_token_id(SHARD_1);

    let token_data = EsdtTokenData {
        amount: BigUint::from(TEN_TOKENS),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(
        EgldOrEsdtTokenIdentifier::esdt(TokenIdentifier::from_esdt_bytes(sov_token_id.clone())),
        0,
        token_data,
    );

    let mvx_esdt_safe_address = chain_interactor
        .common_state
        .get_mvx_esdt_safe_address(SHARD_1)
        .clone();

    let operation_data = OperationData::new(
        chain_interactor
            .common_state()
            .get_and_increment_operation_nonce(&mvx_esdt_safe_address.to_string()),
        ManagedAddress::from_address(&chain_interactor.user_address),
        None,
    );

    let operation = Operation::new(
        SOVEREIGN_RECEIVER_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor
        .register_operation(SHARD_1, &hash_of_hashes, operations_hashes)
        .await;

    let expected_operation_hash_status = OperationHashStatus::NotLocked;
    chain_interactor
        .check_registered_operation_status(
            SHARD_1,
            &hash_of_hashes,
            operation_hash,
            expected_operation_hash_status,
        )
        .await;

    let bridge_service = chain_interactor
        .get_bridge_service_for_shard(SHARD_1)
        .clone();
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            bridge_service,
            SHARD_1,
            hash_of_hashes,
            operation,
            Some(MULTI_ESDT_NFT_TRANSFER_EVENT),
            None,
        )
        .await;

    // Reset the nonce to the previous value
    let current_nonce = chain_interactor
        .common_state()
        .get_operation_nonce(&mvx_esdt_safe_address.to_string());

    chain_interactor
        .common_state()
        .set_operation_nonce(&mvx_esdt_safe_address.to_string(), current_nonce - 1);
}

/// ### TEST
/// M-ESDT_SWITCH_MECHANISM_OK
///
/// ### ACTION
/// Deposit a trusted token into the MVX ESDT Safe with burn mechanism set up and switch to lock mechanism
///
/// ### EXPECTED
/// The token is burned and the deposited amount is tracked in storage, then the mechanism is switched
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_switch_mechanism_with_deposit() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee_wrapper(SHARD_0).await;

    let trusted_token = chain_interactor.common_state().get_trusted_token();
    let trusted_token_id = EgldOrEsdtTokenIdentifier::esdt(trusted_token.as_str());
    let trusted_token_info = EsdtTokenInfo {
        token_id: trusted_token_id.clone(),
        amount: BigUint::from(ONE_HUNDRED_TOKENS),
        nonce: 0,
        decimals: 18,
        token_type: EsdtTokenType::Fungible,
    };

    chain_interactor
        .set_token_burn_mechanism(trusted_token_id.clone(), SHARD_0)
        .await;

    let deposit_amount = BigUint::from(ONE_HUNDRED_TOKENS);
    let esdt_token_payment = EgldOrEsdtTokenPayment::<StaticApi>::new(
        trusted_token_id.clone(),
        0,
        deposit_amount.clone(),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment]);

    let expected_logs = chain_interactor.build_expected_deposit_log(
        ActionConfig::new().shard(SHARD_0),
        Some(trusted_token_info.clone()),
    );
    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_0,
            OptionalValue::None,
            payments_vec.clone(),
            None,
            Some(expected_logs.clone()),
        )
        .await;

    chain_interactor
        .common_state()
        .add_to_deposited_amount(deposit_amount.clone());

    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_0)
        .token(Some(trusted_token_info.clone()))
        .amount(deposit_amount.clone())
        .is_burn_mechanism_set(true);

    chain_interactor
        .check_balances_after_action(balance_config)
        .await;

    chain_interactor
        .check_deposited_tokens_amount(trusted_token_id.clone(), SHARD_0, deposit_amount.clone())
        .await;

    // === Switch to Lock Mechanism ===

    chain_interactor
        .set_token_lock_mechanism(trusted_token_id.clone(), SHARD_0)
        .await;

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_0,
            OptionalValue::None,
            payments_vec,
            None,
            Some(expected_logs),
        )
        .await;

    chain_interactor
        .common_state()
        .add_to_deposited_amount(deposit_amount.clone());

    // Since the mechanism was switched, the trusted token amount was minted in the sc, now we check for both the mint and the new deposit amount
    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_0)
        .token(Some(trusted_token_info))
        .amount(BigUint::from(2u64) * deposit_amount);

    chain_interactor
        .check_balances_after_action(balance_config)
        .await;

    chain_interactor
        .check_deposited_tokens_amount(trusted_token_id.clone(), SHARD_0, BigUint::zero())
        .await;
}

/// ### TEST
/// M-ESDT_EXEC_WITH_BURN_MECHANISM_OK
///
/// ### ACTION
/// Execute an operation with a trusted token with burn mechanism set up
///
/// ### EXPECTED
/// The operation is executed successfully and the deposited amount is updated in storage
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_with_burn_mechanism() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee_wrapper(SHARD_0).await;

    let trusted_token = chain_interactor.common_state().get_trusted_token();
    let trusted_token_id = EgldOrEsdtTokenIdentifier::esdt(trusted_token.as_str());
    let trusted_token_info = EsdtTokenInfo {
        token_id: trusted_token_id.clone(),
        amount: BigUint::from(ONE_HUNDRED_TOKENS),
        nonce: 0,
        decimals: 18,
        token_type: EsdtTokenType::Fungible,
    };

    chain_interactor
        .set_token_burn_mechanism(trusted_token_id.clone(), SHARD_0)
        .await;

    let deposit_amount = BigUint::from(ONE_HUNDRED_TOKENS);
    let esdt_token_payment = EgldOrEsdtTokenPayment::<StaticApi>::new(
        trusted_token_id.clone(),
        0,
        deposit_amount.clone(),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment]);

    let expected_logs = chain_interactor.build_expected_deposit_log(
        ActionConfig::new().shard(SHARD_0),
        Some(trusted_token_info.clone()),
    );
    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_0,
            OptionalValue::None,
            payments_vec.clone(),
            None,
            Some(expected_logs),
        )
        .await;

    chain_interactor
        .common_state()
        .add_to_deposited_amount(deposit_amount.clone());

    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_0)
        .token(Some(trusted_token_info.clone()))
        .amount(deposit_amount.clone())
        .is_burn_mechanism_set(true);

    chain_interactor
        .check_balances_after_action(balance_config)
        .await;

    let current_deposited_amount = chain_interactor.common_state().get_deposited_amount();

    chain_interactor
        .check_deposited_tokens_amount(trusted_token_id.clone(), SHARD_0, current_deposited_amount)
        .await;

    let operation = chain_interactor
        .prepare_operation(
            SHARD_0,
            Some(trusted_token_info.clone()),
            Some(TESTING_SC_ENDPOINT),
        )
        .await;

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor
        .register_operation(SHARD_0, &hash_of_hashes, operations_hashes)
        .await;

    let bridge_service = chain_interactor
        .get_bridge_service_for_shard(SHARD_0)
        .clone();

    let expected_logs = chain_interactor.build_expected_execute_log(
        ActionConfig::new().shard(SHARD_0),
        Some(trusted_token_info.clone()),
    );
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            bridge_service,
            SHARD_0,
            hash_of_hashes,
            operation,
            None,
            Some(expected_logs),
        )
        .await;

    chain_interactor
        .common_state()
        .subtract_from_deposited_amount(deposit_amount.clone());

    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_0)
        .token(Some(trusted_token_info))
        .amount(deposit_amount.clone())
        .is_execute(true)
        .with_transfer_data(true);

    chain_interactor
        .check_balances_after_action(balance_config)
        .await;

    let current_deposited_amount = chain_interactor.common_state().get_deposited_amount();

    chain_interactor
        .check_deposited_tokens_amount(trusted_token_id.clone(), SHARD_0, current_deposited_amount)
        .await;
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_pause_contract() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    chain_interactor.remove_fee_wrapper(SHARD_1).await;

    chain_interactor.switch_pause_status(true, SHARD_1).await;

    chain_interactor.switch_pause_status(false, SHARD_1).await;
}
