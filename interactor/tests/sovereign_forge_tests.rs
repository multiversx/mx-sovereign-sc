use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use common_test_setup::constants::{
    CHAIN_ID, DEPLOY_COST, ESDT_SAFE_CONFIG_STORAGE_KEY, ONE_HUNDRED_TOKENS, ONE_THOUSAND_TOKENS,
    OPERATION_HASH_STATUS_STORAGE_KEY, TEN_TOKENS, TOKEN_FEE_STORAGE_KEY, WRONG_ENDPOINT_NAME,
};
use header_verifier::OperationHashStatus;
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EsdtTokenData, EsdtTokenPayment, ManagedAddress, ManagedBuffer, ManagedVec,
        MultiValueEncoded,
    },
};
use multiversx_sc_snippets::{
    hex,
    imports::{tokio, Bech32Address, StaticApi},
    multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256,
};
use rust_interact::sovereign_forge::sovereign_forge_interactor_main::SovereignForgeInteract;
use serial_test::serial;
use structs::{
    aliases::PaymentsVec,
    configs::EsdtSafeConfig,
    fee::{FeeStruct, FeeType},
    generate_hash::GenerateHash,
    operation::{Operation, OperationData, OperationEsdtPayment, TransferData},
};

/// ### TEST
/// S-FORGE_COMPLETE_SETUP_PHASE_OK
///
/// ### ACTION
/// Run deploy phases 1–4 and call complete_setup_phase
///
/// ### EXPECTED
/// Setup phase is complete
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deploy_sovereign_forge_cs() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    chain_interactor
        .deploy_and_complete_setup_phase(
            CHAIN_ID,
            BigUint::from(DEPLOY_COST),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;
}

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call deposit_in_mvx_esdt_safe
///
/// ### EXPECTED
/// Deposit is successful and tokens are transferred to the mvx-esdt-safe-sc
#[tokio::test]
#[ignore]
// #[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_deposit_flow() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let deploy_cost = BigUint::from(DEPLOY_COST);
    let user_address = chain_interactor.user_address().clone();

    chain_interactor
        .deploy_and_complete_setup_phase(
            CHAIN_ID,
            deploy_cost,
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_second_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            user_address,
            OptionalValue::None,
            payments_vec,
            None,
            Some("deposit"),
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_first_token_id_string(),
            ONE_THOUSAND_TOKENS - ONE_HUNDRED_TOKENS,
        ),
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_second_token_id_string(),
            ONE_THOUSAND_TOKENS - ONE_HUNDRED_TOKENS,
        ),
    ];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address()),
            expected_tokens_wallet,
        )
        .await;

    let expected_tokens_contract = vec![
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_first_token_id_string(),
            ONE_HUNDRED_TOKENS,
        ),
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_second_token_id_string(),
            ONE_HUNDRED_TOKENS,
        ),
    ];
    chain_interactor
        .check_address_balance(
            &chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone(),
            expected_tokens_contract,
        )
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;
    chain_interactor.check_testing_sc_balance_is_empty().await;
}

/// ### TEST
/// S-FORGE_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation(contains transfer data) in a complete flow
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[ignore]
// #[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_flow_execute_operation_with_transfer_data_success_no_fee() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let user_address = chain_interactor.user_address().clone();
    let token_data = EsdtTokenData {
        amount: BigUint::from(TEN_TOKENS),
        ..Default::default()
    };

    let payment =
        OperationEsdtPayment::new(chain_interactor.state.get_first_token_id(), 0, token_data);
    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: chain_interactor.state.get_first_token_id(),
        token_nonce: 0,
        amount: BigUint::from(TEN_TOKENS),
    });

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        Some(transfer_data),
    );

    chain_interactor
        .deploy_and_complete_setup_phase(
            CHAIN_ID,
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
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .to_address(),
            OptionalValue::None,
            payment_vec,
            None,
            Some("deposit"),
        )
        .await;

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor
        .register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes)
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
            hash_of_hashes,
            operation,
            None,
            Some("executedBridgeOp"),
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

    let expected_tokens_wallet = vec![
        (
            chain_interactor.state.get_first_token_id().to_string(),
            BigUint::from(ONE_THOUSAND_TOKENS - TEN_TOKENS),
        ),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(&Bech32Address::from(user_address), expected_tokens_wallet)
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;

    let expected_testing_sc_balance = vec![(
        chain_interactor.state.get_first_token_id().to_string(),
        BigUint::from(TEN_TOKENS),
    )];
    chain_interactor
        .check_address_balance(
            &chain_interactor.state.current_testing_sc_address().clone(),
            expected_testing_sc_balance,
        )
        .await;
}

/// ### TEST
/// S-FORGE_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation(contains transfer data) in a complete flow on both chains
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
/// The fee is deducted
#[tokio::test]
#[serial]
#[ignore]
// #[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_flow_execute_operation_success_with_fee() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let user_address = chain_interactor.user_address().clone();
    let token_data = EsdtTokenData {
        amount: BigUint::from(TEN_TOKENS),
        ..Default::default()
    };

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);
    let fee = FeeStruct {
        base_token: chain_interactor.state.get_fee_token_id(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_fee_token_id(),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function.clone(), args);

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        Some(transfer_data),
    );

    let fee_amount = per_transfer + (per_gas * BigUint::from(gas_limit));

    let payment =
        OperationEsdtPayment::new(chain_interactor.state.get_first_token_id(), 0, token_data);
    let mut payment_vec = PaymentsVec::new();
    let fee_payment = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_fee_token_id(),
        0,
        fee_amount.clone(),
    );

    payment_vec.push(fee_payment);
    payment_vec.push(EsdtTokenPayment {
        token_identifier: chain_interactor.state.get_first_token_id(),
        token_nonce: 0,
        amount: BigUint::from(TEN_TOKENS),
    });

    chain_interactor
        .deploy_and_complete_setup_phase(
            CHAIN_ID,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            Some(fee),
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

    let deposit_args = MultiValueEncoded::from(ManagedVec::from(vec![ManagedBuffer::from("1")]));
    let deposit_transfer_data = MultiValue3::from((gas_limit, function, deposit_args));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .to_address(),
            OptionalValue::Some(deposit_transfer_data),
            payment_vec,
            None,
            Some("deposit"),
        )
        .await;

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor
        .register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes)
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
            hash_of_hashes,
            operation,
            None,
            Some("executedBridgeOp"),
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

    let expected_tokens_wallet = vec![
        (
            chain_interactor.state.get_first_token_id().to_string(),
            BigUint::from(ONE_THOUSAND_TOKENS - TEN_TOKENS),
        ),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        (
            chain_interactor.state.get_fee_token_id().to_string(),
            BigUint::from(ONE_THOUSAND_TOKENS) - fee_amount.clone(),
        ),
    ];
    chain_interactor
        .check_address_balance(&Bech32Address::from(user_address), expected_tokens_wallet)
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    let expected_token_fee_market = vec![(
        chain_interactor.state.get_fee_token_id().to_string(),
        fee_amount,
    )];
    chain_interactor
        .check_address_balance(
            &chain_interactor.state.current_fee_market_address().clone(),
            expected_token_fee_market,
        )
        .await;

    let expected_testing_sc_balance = vec![(
        chain_interactor.state.get_first_token_id().to_string(),
        BigUint::from(TEN_TOKENS),
    )];
    chain_interactor
        .check_address_balance(
            &chain_interactor.state.current_testing_sc_address().clone(),
            expected_testing_sc_balance,
        )
        .await;
}

/// ### TEST
/// S-FORGE_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation(contains transfer data) and no fee
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[ignore]
// #[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_flow_execute_operation_only_transfer_data_no_fee() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        Some(transfer_data),
    );

    chain_interactor
        .deploy_and_complete_setup_phase(
            CHAIN_ID,
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
        .register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes)
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
            hash_of_hashes,
            operation,
            None,
            Some("executedBridgeOp"),
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

    chain_interactor.check_wallet_balance_unchanged().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;
    chain_interactor.check_testing_sc_balance_is_empty().await;
}

/// ### TEST
/// S-FORGE_EXEC_FAIL
///
/// ### ACTION
/// Call 'execute_operation()' with invalid endpoint in transfer data
///
/// ### EXPECTED
/// The testing smart contract returns a failed event
#[tokio::test]
#[serial]
#[ignore]
// #[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_flow_execute_operation_wrong_endpoint() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

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
        .deploy_and_complete_setup_phase(
            CHAIN_ID,
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
        .register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes)
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
            hash_of_hashes,
            operation,
            Some(function.to_string().as_str()),
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

    chain_interactor.check_wallet_balance_unchanged().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;
    chain_interactor.check_testing_sc_balance_is_empty().await;
}

/// ### TEST
/// S-FORGE_UPDATE_ESDT_SAFE_CONFIG_OK
///     
/// ### ACTION
/// Deploy and complete setup phase, then call update_esdt_safe_config
///
/// ### EXPECTED
/// The ESDT Safe config is updated successfully
#[tokio::test]
#[serial]
#[ignore]
// #[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_flow_update_esdt_safe_config() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            CHAIN_ID,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let new_esdt_safe_config = EsdtSafeConfig::new(
        ManagedVec::from_single_item(chain_interactor.state.get_first_token_id()),
        ManagedVec::from_single_item(chain_interactor.state.get_second_token_id()),
        120_000_000u64,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    let config_hash = new_esdt_safe_config.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&config_hash.to_vec()));

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![config_hash.clone()]));

    chain_interactor
        .register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes)
        .await;

    chain_interactor
        .update_esdt_safe_config(hash_of_hashes, new_esdt_safe_config)
        .await;

    let wanted_key_encoded = hex::encode(ESDT_SAFE_CONFIG_STORAGE_KEY);
    let expected_value_encoded = hex::encode(chain_interactor.state.get_first_token_id_string());
    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            wanted_key_encoded.as_str(),
            Some(&expected_value_encoded),
        )
        .await;
}

/// ### TEST
/// S-FORGE_SET_AND_REMOVE_FEE_OK
///     
/// ### ACTION
/// Deploy and complete setup phase, then call set_fee and remove_fee
///
/// ### EXPECTED
/// The fee is set and then removed successfully
#[tokio::test]
#[serial]
#[ignore]
// #[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_flow_set_and_remove_fee() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_and_complete_setup_phase(
            CHAIN_ID,
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let fee = FeeStruct {
        base_token: chain_interactor.state.get_fee_token_id(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_fee_token_id(),
            per_transfer: BigUint::from(100u64),
            per_gas: BigUint::from(1u64),
        },
    };

    let fee_hash = fee.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&fee_hash.to_vec()));

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![fee_hash.clone()]));

    chain_interactor
        .register_operation(
            ManagedBuffer::new(),
            &hash_of_hashes,
            operations_hashes.clone(),
        )
        .await;

    chain_interactor
        .set_fee_after_setup_phase(hash_of_hashes.clone(), fee)
        .await;

    let wanted_key_encoded = hex::encode(TOKEN_FEE_STORAGE_KEY);
    let expected_value_encoded = hex::encode(chain_interactor.state.get_fee_token_id_string());
    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_fee_market_address()
                .clone()
                .to_address(),
            wanted_key_encoded.as_str(),
            Some(&expected_value_encoded),
        )
        .await;

    let remove_fee_hash = sha256(
        &chain_interactor
            .state
            .get_fee_token_id()
            .as_managed_buffer()
            .to_vec(),
    );

    let remove_fee_hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&remove_fee_hash));

    chain_interactor
        .register_operation(
            ManagedBuffer::new(),
            &remove_fee_hash_of_hashes,
            MultiValueEncoded::from_iter(vec![ManagedBuffer::new_from_bytes(&remove_fee_hash)]),
        )
        .await;

    chain_interactor
        .remove_fee_after_setup_phase(
            remove_fee_hash_of_hashes,
            chain_interactor.state.get_fee_token_id(),
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_fee_market_address()
                .clone()
                .to_address(),
            wanted_key_encoded.as_str(),
            None,
        )
        .await;
}
