use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_test_setup::constants::{
    DEPLOY_COST, ONE_HUNDRED_TOKENS, ONE_THOUSAND_TOKENS, OPERATION_HASH_STATUS_STORAGE_KEY,
    SHARD_0, SHARD_2, TESTING_SC_ENDPOINT,
};
use header_verifier::OperationHashStatus;
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EsdtTokenData, EsdtTokenPayment, ManagedAddress, ManagedBuffer, ManagedVec,
        MultiValueEncoded, TokenIdentifier,
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
    fee::{FeeStruct, FeeType},
    operation::{Operation, OperationData, OperationEsdtPayment, TransferData},
};

//TODO: Change expected log to be DEPOSIT_LOG and EXECUTED_BRIDGE_LOG when the framework fix is implemented

/// ### TEST
/// S-FORGE_COMPLETE-DEPOSIT-FLOW_OK
///
/// ### ACTION
/// Deploy and complete setup phase, then call deposit_in_mvx_esdt_safe
///
/// ### EXPECTED
/// Deposit is successful and tokens are transferred to the mvx-esdt-safe-sc
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_deposit_flow_no_fee_different_shard() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_2;

    let deploy_cost = BigUint::from(DEPLOY_COST);

    chain_interactor
        .deploy_and_complete_setup_phase(
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
            chain_interactor
                .state
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::None,
            payments_vec,
            None,
            Some(&chain_interactor.state.get_first_token_id_string()),
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
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
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
                .get_mvx_esdt_safe_address(shard)
                .clone(),
            expected_tokens_contract,
        )
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
/// Deploy and complete setup phase, then call deposit_in_mvx_esdt_safe
///
/// ### EXPECTED
/// Deposit is successful and tokens are transferred to the mvx-esdt-safe-sc
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_deposit_flow_only_transfer_data_no_fee_different_shard() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_2;

    let deploy_cost = BigUint::from(DEPLOY_COST);

    chain_interactor
        .deploy_and_complete_setup_phase(
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

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));
    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            chain_interactor
                .state
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::Some(transfer_data),
            payments_vec,
            None,
            Some(&chain_interactor.state.get_first_token_id_string()),
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
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
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
                .get_mvx_esdt_safe_address(shard)
                .clone(),
            expected_tokens_contract,
        )
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;
    chain_interactor.check_testing_sc_balance_is_empty().await;
}

/// ### TEST
/// S-FORGE_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_success_no_fee_different_shard() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
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
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

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
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    let caller = chain_interactor.get_bridge_service_for_shard(shard);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(""),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let additional_expected_tokens_wallet = vec![
        chain_interactor.zero_tokens(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
    ];
    chain_interactor
        .check_wallet_balance_unchanged(Some(additional_expected_tokens_wallet))
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;
}

/// ### TEST
/// S-FORGE_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' to transfer a nft with no fee
///
/// ### EXPECTED
/// The operation is executed and the tokens are received in the expected wallet
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_success_no_fee_different_shard_transfer_nft() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_2;

    let token_data = EsdtTokenData {
        amount: BigUint::from(1u64),
        ..Default::default()
    };

    let nft_token = chain_interactor.state.get_nft_token_id();
    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from_esdt_bytes(nft_token.clone().token_id),
        nft_token.nonce,
        token_data,
    );

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        None,
    );

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from_esdt_bytes(nft_token.token_id.clone()),
        token_nonce: nft_token.nonce,
        amount: BigUint::from(1u64),
    });

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            chain_interactor
                .state
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::None,
            payment_vec,
            None,
            Some(&chain_interactor.state.get_nft_token_id_string()),
        )
        .await;

    let operation = Operation::new(
        ManagedAddress::from_address(&chain_interactor.user_address),
        vec![payment].into(),
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
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    let caller = chain_interactor.get_bridge_service_for_shard(shard);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(&chain_interactor.state.get_nft_token_id_string()),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
        chain_interactor.zero_tokens(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_tokens_wallet,
        )
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;
    chain_interactor.check_testing_sc_balance_is_empty().await;
}

/// ### TEST
/// S-FORGE_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' to transfer sfts with no fee
///
/// ### EXPECTED
/// The operation is executed and the tokens are received in the expected wallet
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_success_no_fee_different_shard_transfer_sft() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    let token_data = EsdtTokenData {
        amount: BigUint::from(ONE_HUNDRED_TOKENS),
        ..Default::default()
    };

    let sft_token = chain_interactor.state.get_sft_token_id();
    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from_esdt_bytes(sft_token.clone().token_id),
        sft_token.nonce,
        token_data,
    );

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        None,
    );

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let operation = Operation::new(
        ManagedAddress::from_address(&chain_interactor.user_address),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from_esdt_bytes(sft_token.token_id.clone()),
        token_nonce: sft_token.nonce,
        amount: BigUint::from(ONE_HUNDRED_TOKENS),
    });

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            chain_interactor
                .state
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::None,
            payment_vec,
            None,
            Some(&chain_interactor.state.get_sft_token_id_string()),
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
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    let caller = chain_interactor.get_bridge_service_for_shard(shard);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(&chain_interactor.state.get_sft_token_id_string()),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_tokens_wallet,
        )
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;
    chain_interactor.check_testing_sc_balance_is_empty().await;
}

/// ### TEST
/// S-FORGE_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' to transfer meta-esdts with no fee
///
/// ### EXPECTED
/// The operation is executed and the tokens are received in the expected wallet
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_success_no_fee_different_shard_transfer_meta_esdt() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_2;

    let token_data = EsdtTokenData {
        amount: BigUint::from(ONE_HUNDRED_TOKENS),
        ..Default::default()
    };

    let meta_esdt_token = chain_interactor.state.get_meta_esdt_token_id();
    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from_esdt_bytes(meta_esdt_token.clone().token_id),
        meta_esdt_token.nonce,
        token_data,
    );

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        None,
    );

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let operation = Operation::new(
        ManagedAddress::from_address(&chain_interactor.user_address),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from_esdt_bytes(meta_esdt_token.token_id.clone()),
        token_nonce: meta_esdt_token.nonce,
        amount: BigUint::from(ONE_HUNDRED_TOKENS),
    });

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            chain_interactor
                .state
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::None,
            payment_vec,
            None,
            Some(&chain_interactor.state.get_meta_esdt_token_id_string()),
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
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    let caller = chain_interactor.get_bridge_service_for_shard(shard);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(&chain_interactor.state.get_meta_esdt_token_id_string()),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_tokens_wallet,
        )
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;
    chain_interactor.check_testing_sc_balance_is_empty().await;
}

/// ### TEST
/// S-FORGE_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' to transfer a dynamic NFT with no fee
///
/// ### EXPECTED
/// The operation is executed and the tokens are received in the expected wallet
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_success_no_fee_different_shard_transfer_dynamic_nft() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    let token_data = EsdtTokenData {
        amount: BigUint::from(1u64),
        ..Default::default()
    };

    let dynamic_nft: common_interactor::interactor_state::TokenProperties =
        chain_interactor.state.get_dynamic_nft_token_id();
    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from_esdt_bytes(dynamic_nft.clone().token_id),
        dynamic_nft.nonce,
        token_data,
    );

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        None,
    );

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

    let operation = Operation::new(
        ManagedAddress::from_address(&chain_interactor.user_address),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from_esdt_bytes(dynamic_nft.token_id.clone()),
        token_nonce: dynamic_nft.nonce,
        amount: BigUint::from(1u64),
    });

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            chain_interactor
                .state
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::None,
            payment_vec,
            None,
            Some(&chain_interactor.state.get_dynamic_nft_token_id_string()),
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
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    let caller = chain_interactor.get_bridge_service_for_shard(shard);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(&chain_interactor.state.get_dynamic_nft_token_id_string()),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.zero_tokens(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_tokens_wallet,
        )
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;
    chain_interactor.check_testing_sc_balance_is_empty().await;
    let expected_second_user_balance =
        vec![chain_interactor.one_token(dynamic_nft.token_id.clone())];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.user_address.clone()),
            expected_second_user_balance,
        )
        .await;
}

/// ### TEST
/// S-FORGE_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' to transfer a dynamic NFT with a fee
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
/// The fee is deducted
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_execute_operation_success_with_fee_different_shard_transfer_dynamic_nft() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_2;

    let user_address = chain_interactor.user_address().clone();
    let token_data = EsdtTokenData {
        amount: BigUint::from(1u64),
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

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.user_address),
        None,
    );

    let fee_amount = per_transfer;

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from_esdt_bytes(
            chain_interactor
                .state
                .get_dynamic_nft_token_id()
                .token_id
                .clone(),
        ),
        chain_interactor.state.get_dynamic_nft_token_id().nonce,
        token_data,
    );
    let mut payment_vec = PaymentsVec::new();
    let fee_payment = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_fee_token_id(),
        0,
        fee_amount.clone(),
    );

    payment_vec.push(fee_payment);
    payment_vec.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from_esdt_bytes(
            chain_interactor
                .state
                .get_dynamic_nft_token_id()
                .token_id
                .clone(),
        ),
        token_nonce: chain_interactor.state.get_dynamic_nft_token_id().nonce,
        amount: BigUint::from(1u64),
    });

    chain_interactor
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            Some(fee),
        )
        .await;

    let operation = Operation::new(
        ManagedAddress::from_address(&chain_interactor.user_address),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    chain_interactor
        .deposit_in_mvx_esdt_safe(
            chain_interactor
                .state
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::None,
            payment_vec,
            None,
            Some(&chain_interactor.state.get_dynamic_nft_token_id_string()),
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
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    let caller = chain_interactor.get_bridge_service_for_shard(shard);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(&chain_interactor.state.get_dynamic_nft_token_id_string()),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_fee_token_id_string(),
            BigUint::from(ONE_THOUSAND_TOKENS) - fee_amount.clone(),
        ),
        chain_interactor.one_token(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.zero_tokens(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(&Bech32Address::from(user_address), expected_tokens_wallet)
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    let expected_token_fee_market = vec![(
        chain_interactor.state.get_fee_token_id().to_string(),
        fee_amount,
    )];
    chain_interactor
        .check_address_balance(
            &chain_interactor.state.get_fee_market_address(shard).clone(),
            expected_token_fee_market,
        )
        .await;
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
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_flow_execute_operation_with_transfer_data_success_nft_no_fee() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    let user_address = chain_interactor.user_address().clone();
    let token_data = EsdtTokenData {
        amount: BigUint::from(1u64),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from_esdt_bytes(chain_interactor.state.get_nft_token_id().token_id),
        chain_interactor.state.get_nft_token_id().nonce,
        token_data,
    );
    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from_esdt_bytes(
            chain_interactor.state.get_nft_token_id().token_id,
        ),
        token_nonce: chain_interactor.state.get_nft_token_id().nonce,
        amount: BigUint::from(1u64),
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
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

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
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::None,
            payment_vec,
            None,
            Some(&chain_interactor.state.get_nft_token_id_string()),
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
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    let caller = chain_interactor.get_bridge_service_for_shard(shard);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(chain_interactor.state.get_nft_token_id_string().as_str()),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
        chain_interactor.zero_tokens(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(&Bech32Address::from(user_address), expected_tokens_wallet)
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;

    let expected_testing_sc_balance = vec![(
        chain_interactor.state.get_nft_token_id_string(),
        BigUint::from(1u64),
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
/// Call 'execute_operation()' with valid operation(contains transfer data) in a complete flow
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_flow_execute_operation_with_transfer_data_success_sft_no_fee() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    let user_address = chain_interactor.user_address().clone();
    let token_data = EsdtTokenData {
        amount: BigUint::from(ONE_HUNDRED_TOKENS),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from_esdt_bytes(chain_interactor.state.get_sft_token_id().token_id),
        chain_interactor.state.get_sft_token_id().nonce,
        token_data,
    );
    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from_esdt_bytes(
            chain_interactor.state.get_sft_token_id().token_id,
        ),
        token_nonce: chain_interactor.state.get_sft_token_id().nonce,
        amount: BigUint::from(ONE_HUNDRED_TOKENS),
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
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

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
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::None,
            payment_vec,
            None,
            Some(&chain_interactor.state.get_sft_token_id_string()),
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
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    let caller = chain_interactor.get_bridge_service_for_shard(shard);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(chain_interactor.state.get_sft_token_id_string().as_str()),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_sft_token_id_string(),
            ONE_THOUSAND_TOKENS - ONE_HUNDRED_TOKENS,
        ),
    ];
    chain_interactor
        .check_address_balance(&Bech32Address::from(user_address), expected_tokens_wallet)
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;

    let expected_testing_sc_balance = vec![(
        chain_interactor.state.get_sft_token_id_string(),
        BigUint::from(ONE_HUNDRED_TOKENS),
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
/// Call 'execute_operation()' with valid operation(contains transfer data) in a complete flow
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_flow_execute_operation_with_transfer_data_success_meta_esdt_no_fee() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    let user_address = chain_interactor.user_address().clone();
    let token_data = EsdtTokenData {
        amount: BigUint::from(ONE_HUNDRED_TOKENS),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from_esdt_bytes(chain_interactor.state.get_meta_esdt_token_id().token_id),
        chain_interactor.state.get_meta_esdt_token_id().nonce,
        token_data,
    );
    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from_esdt_bytes(
            chain_interactor.state.get_meta_esdt_token_id().token_id,
        ),
        token_nonce: chain_interactor.state.get_meta_esdt_token_id().nonce,
        amount: BigUint::from(ONE_HUNDRED_TOKENS),
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
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

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
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::None,
            payment_vec,
            None,
            Some(&chain_interactor.state.get_meta_esdt_token_id_string()),
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
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    let caller = chain_interactor.get_bridge_service_for_shard(shard);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(
                chain_interactor
                    .state
                    .get_meta_esdt_token_id_string()
                    .as_str(),
            ),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_meta_esdt_token_id_string(),
            ONE_THOUSAND_TOKENS - ONE_HUNDRED_TOKENS,
        ),
        chain_interactor.one_token(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(&Bech32Address::from(user_address), expected_tokens_wallet)
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;

    let expected_testing_sc_balance = vec![(
        chain_interactor.state.get_meta_esdt_token_id_string(),
        BigUint::from(ONE_HUNDRED_TOKENS),
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
/// Call 'execute_operation()' with valid operation(contains transfer data) in a complete flow
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_flow_execute_operation_with_transfer_data_success_dynamic_nft_no_fee() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    let user_address = chain_interactor.user_address().clone();
    let token_data = EsdtTokenData {
        amount: BigUint::from(1u64),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from_esdt_bytes(
            chain_interactor.state.get_dynamic_nft_token_id().token_id,
        ),
        chain_interactor.state.get_dynamic_nft_token_id().nonce,
        token_data,
    );
    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from_esdt_bytes(
            chain_interactor.state.get_dynamic_nft_token_id().token_id,
        ),
        token_nonce: chain_interactor.state.get_dynamic_nft_token_id().nonce,
        amount: BigUint::from(1u64),
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
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

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
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::None,
            payment_vec,
            None,
            Some(&chain_interactor.state.get_dynamic_nft_token_id_string()),
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
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    let caller = chain_interactor.get_bridge_service_for_shard(shard);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(
                chain_interactor
                    .state
                    .get_dynamic_nft_token_id_string()
                    .as_str(),
            ),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
        chain_interactor.one_token(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.zero_tokens(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(&Bech32Address::from(user_address), expected_tokens_wallet)
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;
    chain_interactor
        .check_fee_market_balance_is_empty(shard)
        .await;

    let expected_testing_sc_balance = vec![(
        chain_interactor.state.get_dynamic_nft_token_id_string(),
        BigUint::from(1u64),
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
/// Call 'execute_operation()' with valid operation(contains transfer data) in a complete flow
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_complete_flow_execute_operation_with_transfer_data_success_dynamic_nft_with_fee() {
    let mut chain_interactor = SovereignForgeInteract::new(Config::chain_simulator_config()).await;
    let shard = SHARD_0;

    let user_address = chain_interactor.user_address().clone();
    let token_data = EsdtTokenData {
        amount: BigUint::from(1u64),
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

    let fee_amount = per_transfer;

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from_esdt_bytes(
            chain_interactor.state.get_dynamic_nft_token_id().token_id,
        ),
        chain_interactor.state.get_dynamic_nft_token_id().nonce,
        token_data,
    );

    let mut payment_vec = PaymentsVec::new();
    let fee_payment = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_fee_token_id(),
        0,
        fee_amount.clone(),
    );
    payment_vec.push(fee_payment);
    payment_vec.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from_esdt_bytes(
            chain_interactor.state.get_dynamic_nft_token_id().token_id,
        ),
        token_nonce: chain_interactor.state.get_dynamic_nft_token_id().nonce,
        amount: BigUint::from(1u64),
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
        .deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            Some(fee),
        )
        .await;

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
                .get_mvx_esdt_safe_address(shard)
                .to_address(),
            shard,
            OptionalValue::None,
            payment_vec,
            None,
            Some(&chain_interactor.state.get_dynamic_nft_token_id_string()),
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
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    let caller = chain_interactor.get_bridge_service_for_shard(shard);
    chain_interactor
        .execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            None,
            Some(
                chain_interactor
                    .state
                    .get_dynamic_nft_token_id_string()
                    .as_str(),
            ),
            None,
        )
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .get_header_verifier_address(shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_fee_token_id_string(),
            BigUint::from(ONE_THOUSAND_TOKENS) - fee_amount.clone(),
        ),
        chain_interactor.one_token(chain_interactor.state.get_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_meta_esdt_token_id_string()),
        chain_interactor.zero_tokens(chain_interactor.state.get_dynamic_nft_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_sft_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(&Bech32Address::from(user_address), expected_tokens_wallet)
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty(shard)
        .await;

    let expected_fee_market_balance =
        vec![(chain_interactor.state.get_fee_token_id_string(), fee_amount)];
    chain_interactor
        .check_address_balance(
            &chain_interactor.state.get_fee_market_address(shard).clone(),
            expected_fee_market_balance,
        )
        .await;

    let expected_testing_sc_balance = vec![(
        chain_interactor.state.get_dynamic_nft_token_id_string(),
        BigUint::from(1u64),
    )];
    chain_interactor
        .check_address_balance(
            &chain_interactor.state.current_testing_sc_address().clone(),
            expected_testing_sc_balance,
        )
        .await;
}
