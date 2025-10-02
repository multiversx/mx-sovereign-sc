use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_interactor::interactor_helpers::InteractorHelpers;
use common_interactor::interactor_structs::BalanceCheckConfig;
use common_test_setup::constants::{
    DEPOSIT_EVENT, EXECUTED_BRIDGE_LOG, EXECUTED_BRIDGE_OP_EVENT, GAS_LIMIT, ONE_HUNDRED_TOKENS,
    PER_GAS, PER_TRANSFER, SC_CALL_EVENT, SHARD_0, SOVEREIGN_RECEIVER_ADDRESS, TEN_TOKENS,
    TESTING_SC_ENDPOINT,
};
use cross_chain::MAX_GAS_PER_TRANSACTION;
use error_messages::{
    BANNED_ENDPOINT_NAME, CURRENT_OPERATION_NOT_REGISTERED, DEPOSIT_OVER_MAX_AMOUNT,
    ERR_EMPTY_PAYMENTS, GAS_LIMIT_TOO_HIGH, MAX_GAS_LIMIT_PER_TX_EXCEEDED, NOTHING_TO_TRANSFER,
    TOO_MANY_TOKENS,
};
use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use rust_interact::mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use serial_test::serial;
use std::vec;
use structs::aliases::PaymentsVec;
use structs::configs::{EsdtSafeConfig, MaxBridgedAmount};
use structs::operation::{Operation, OperationData, OperationEsdtPayment, TransferData};
use structs::OperationHashStatus;

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

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        MAX_GAS_PER_TRANSACTION + 1,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    chain_interactor
        .update_configuration_after_setup_phase(
            SHARD_0,
            config,
            Some(EXECUTED_BRIDGE_OP_EVENT),
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

    chain_interactor.remove_fee(SHARD_0).await;

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from(TESTING_SC_ENDPOINT)]),
        ManagedVec::from(vec![MaxBridgedAmount {
            token_id: chain_interactor.state.get_first_fungible_token_identifier(),
            amount: BigUint::default(),
        }]),
    );

    chain_interactor
        .update_configuration_after_setup_phase(SHARD_0, config, Some(EXECUTED_BRIDGE_LOG), None)
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
            SHARD_0,
            OptionalValue::None,
            payments_vec,
            Some(DEPOSIT_OVER_MAX_AMOUNT),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_0).await;

    chain_interactor
        .update_configuration_after_setup_phase(
            SHARD_0,
            EsdtSafeConfig::default_config(),
            Some(EXECUTED_BRIDGE_LOG),
            None,
        )
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

    chain_interactor.remove_fee(SHARD_0).await;

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_0,
            OptionalValue::None,
            ManagedVec::new(),
            Some(NOTHING_TO_TRANSFER),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_0).await;
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
#[ignore = "This should fail but for now the failing logs are not retrieved by the framework"]
// #[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_too_many_tokens_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee(SHARD_0).await;

    let esdt_token_payment = EgldOrEsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_fungible_token_identifier(),
        0,
        BigUint::from(1u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment; 11]);

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_0,
            OptionalValue::None,
            payments_vec,
            Some(TOO_MANY_TOKENS),
            Some(
                &chain_interactor
                    .state
                    .get_first_fungible_token_identifier()
                    .into_name()
                    .to_string(),
            ),
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_0).await;
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

    chain_interactor.remove_fee(SHARD_0).await;

    let esdt_token_payment_one = EgldOrEsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_fungible_token_identifier(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one]);

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_0,
            OptionalValue::None,
            payments_vec,
            None,
            Some(DEPOSIT_EVENT),
        )
        .await;

    let first_token_id = chain_interactor.state.get_first_fungible_token_id();

    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_0)
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

    chain_interactor.remove_fee(SHARD_0).await;

    let shard = SHARD_0;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        1,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    chain_interactor
        .update_configuration_after_setup_phase(SHARD_0, config, Some(EXECUTED_BRIDGE_LOG), None)
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
        .update_configuration_after_setup_phase(
            SHARD_0,
            EsdtSafeConfig::default_config(),
            Some(EXECUTED_BRIDGE_LOG),
            None,
        )
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

    chain_interactor.remove_fee(SHARD_0).await;

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from(TESTING_SC_ENDPOINT)]),
        ManagedVec::new(),
    );

    chain_interactor
        .update_configuration_after_setup_phase(SHARD_0, config, Some(EXECUTED_BRIDGE_LOG), None)
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
            SHARD_0,
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some(BANNED_ENDPOINT_NAME),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_0).await;

    chain_interactor
        .update_configuration_after_setup_phase(
            SHARD_0,
            EsdtSafeConfig::default_config(),
            Some(EXECUTED_BRIDGE_LOG),
            None,
        )
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
    chain_interactor.set_fee(fee.clone(), SHARD_0).await;

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

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_0,
            OptionalValue::Some(transfer_data),
            payments_vec.clone(),
            None,
            Some(
                &chain_interactor
                    .state
                    .get_fee_token_identifier()
                    .into_esdt_option()
                    .unwrap()
                    .to_string(),
            ),
        )
        .await;

    let first_token_id = chain_interactor.state.get_first_fungible_token_id();
    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_0)
        .token(Some(first_token_id.clone()))
        .amount(ONE_HUNDRED_TOKENS.into())
        .fee(fee.clone())
        .with_transfer_data(true);

    chain_interactor
        .check_balances_after_action(balance_config)
        .await;
    chain_interactor
        .update_fee_market_balance_state(Some(fee.clone()), payments_vec, SHARD_0)
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

    chain_interactor.set_fee(fee, SHARD_0).await;

    let gas_limit = 1000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_0,
            OptionalValue::Some(transfer_data),
            ManagedVec::new(),
            Some(ERR_EMPTY_PAYMENTS),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_0).await;
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

    chain_interactor.remove_fee(SHARD_0).await;

    let gas_limit = 1000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_0,
            OptionalValue::Some(transfer_data),
            ManagedVec::new(),
            None,
            Some(SC_CALL_EVENT),
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_0).await;
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

    chain_interactor.remove_fee(SHARD_0).await;

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
        .get_bridge_service_for_shard(SHARD_0)
        .clone();

    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            bridge_service,
            SHARD_0,
            hash_of_hashes,
            operation,
            None,
            Some(EXECUTED_BRIDGE_OP_EVENT),
            Some(CURRENT_OPERATION_NOT_REGISTERED),
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_0).await;
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

    chain_interactor.remove_fee(SHARD_0).await;

    let token_data = EsdtTokenData {
        amount: BigUint::from(TEN_TOKENS),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(
        chain_interactor.state.get_first_fungible_token_identifier(),
        0,
        token_data,
    );
    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EgldOrEsdtTokenPayment::new(
        chain_interactor.state.get_first_fungible_token_identifier(),
        0,
        BigUint::from(TEN_TOKENS),
    ));

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

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            SHARD_0,
            OptionalValue::None,
            payment_vec,
            None,
            Some(DEPOSIT_EVENT),
        )
        .await;

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor
        .register_operation(SHARD_0, &hash_of_hashes, operations_hashes)
        .await;

    let expected_operation_hash_status = OperationHashStatus::NotLocked;
    chain_interactor
        .check_registered_operation_status(
            SHARD_0,
            &hash_of_hashes,
            operation_hash,
            expected_operation_hash_status,
        )
        .await;

    let bridge_service = chain_interactor
        .get_bridge_service_for_shard(SHARD_0)
        .clone();
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            bridge_service,
            SHARD_0,
            hash_of_hashes,
            operation,
            None,
            Some(EXECUTED_BRIDGE_LOG),
            None,
        )
        .await;

    let balance_config = BalanceCheckConfig::new()
        .shard(SHARD_0)
        .token(Some(chain_interactor.state.get_first_fungible_token_id()))
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
/// Call 'execute_operation()' with valid operation and no fee
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_only_transfer_data_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor.remove_fee(SHARD_0).await;

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
        .register_operation(SHARD_0, &hash_of_hashes, operations_hashes)
        .await;

    let expected_operation_status = OperationHashStatus::NotLocked;
    chain_interactor
        .check_registered_operation_status(
            SHARD_0,
            &hash_of_hashes,
            operation_hash,
            expected_operation_status,
        )
        .await;

    let bridge_service = chain_interactor
        .get_bridge_service_for_shard(SHARD_0)
        .clone();
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            bridge_service,
            SHARD_0,
            hash_of_hashes,
            operation,
            None,
            Some(EXECUTED_BRIDGE_LOG),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor.check_contracts_empty(SHARD_0).await;
}
