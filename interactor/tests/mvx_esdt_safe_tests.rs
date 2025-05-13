use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_test_setup::constants::{
    FEE_TOKEN, FIRST_TEST_TOKEN, ISSUE_COST, OPERATION_HASH_STATUS_STORAGE_KEY, SECOND_TEST_TOKEN,
    SOV_TOKEN, SOV_TO_MVX_TOKEN_STORAGE_KEY, TOKEN_TICKER,
};
use common_test_setup::RegisterTokenArgs;
use error_messages::{
    BANNED_ENDPOINT_NAME, GAS_LIMIT_TOO_HIGH, INVALID_TYPE, NOTHING_TO_TRANSFER,
    PAYMENT_DOES_NOT_COVER_FEE, SETUP_PHASE_NOT_COMPLETED, TOO_MANY_TOKENS,
};
use header_verifier::OperationHashStatus;
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_snippets::{hex, imports::*};
use proxies::fee_market_proxy::{FeeStruct, FeeType};
use rust_interact::mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use serial_test::serial;
use structs::aliases::PaymentsVec;
use structs::configs::{EsdtSafeConfig, SovereignConfig};
use structs::operation::{Operation, OperationData, OperationEsdtPayment, TransferData};

// Test that deposit fails when there is nothing to transfer and fee is disabled
// Steps:
// 1. Deploy header verifier smart contract
// 2. Deploy mvx-esdt-safe smart contract
// 3. Deploy fee-market smart contract
// 4. Set fee-market address in mvx-esdt-safe smart contract
// 5. Unpause mvx-esdt-safe smart contract
// 6. Deposit tokens
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_nothing_to_transfer_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_contracts(
            SovereignConfig::default_config(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
            None,
        )
        .await;

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            ManagedVec::new(),
            Some(NOTHING_TO_TRANSFER),
            None,
        )
        .await;

    let address_states = vec![chain_interactor.state.current_fee_market_address().clone()];

    chain_interactor
        .reset_state_chain_sim(Some(address_states))
        .await;
}

// Test that deposit fails when the token limit is exceeded and fee is disabled
// Steps:
// 1. Deploy header verifier smart contract
// 2. Deploy mvx-esdt-safe smart contract
// 3. Deploy fee-market smart contract
// 4. Set fee-market address in mvx-esdt-safe smart contract
// 5. Unpause mvx-esdt-safe smart contract
// 6. Create a payments vector consisting of 11 EsdtTokenPayments
// 7. Deposit tokens
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_too_many_tokens_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_contracts(
            SovereignConfig::default_config(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
            None,
        )
        .await;

    let esdt_token_payment = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(1u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment; 11]);

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            payments_vec,
            Some(TOO_MANY_TOKENS),
            None,
        )
        .await;

    let address_states = vec![chain_interactor.state.current_fee_market_address().clone()];

    chain_interactor
        .reset_state_chain_sim(Some(address_states))
        .await;
}

// Test that deposit works when there is no transfer data and fee is disabled
// Steps:
// 1. Deploy header verifier smart contract
// 2. Deploy mvx-esdt-safe smart contract
// 3. Deploy fee-market smart contract
// 4. Set fee-market address in mvx-esdt-safe smart contract
// 5. Unpause mvx-esdt-safe smart contract
// 6. Create a payments vector
// 7. Deposit and check logs
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_no_transfer_data_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_contracts(
            SovereignConfig::default_config(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
            None,
        )
        .await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(1u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FEE_TOKEN),
        0,
        BigUint::from(1u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            payments_vec,
            None,
            Some("deposit"),
        )
        .await;

    let address_states = vec![chain_interactor.state.current_fee_market_address().clone()];

    chain_interactor
        .reset_state_chain_sim(Some(address_states))
        .await;
}

// Test that deposit fails when the gas limit is too high and fee is disabled
// Steps:
// 1. Deploy header verifier smart contract
// 2. Deploy mvx-esdt-safe smart contract
// 3. Deploy fee-market smart contract
// 4. Set fee-market address in mvx-esdt-safe smart contract
// 5. Unpause mvx-esdt-safe smart contract
// 6. Deploy testing smart contract
// 7. Create a payments vector
// 8. Create transfer data
// 9. Deposit
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_gas_limit_too_high_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        1,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    chain_interactor
        .deploy_contracts(
            SovereignConfig::default_config(),
            OptionalValue::Some(config),
            None,
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some(GAS_LIMIT_TOO_HIGH),
            None,
        )
        .await;

    let address_states = vec![
        chain_interactor.state.current_fee_market_address().clone(),
        chain_interactor.state.current_testing_sc_address().clone(),
    ];

    chain_interactor
        .reset_state_chain_sim(Some(address_states))
        .await;
}

// Test that deposit fails when the endpoint is banned and fee is disabled
// Steps:
// 1. Create an EsdtSafeConfig with a banned endpoint
// 2. Deploy header verifier smart contract
// 3. Deploy mvx-esdt-safe smart contract
// 4. Deploy fee-market smart contract
// 5. Set fee-market address in mvx-esdt-safe smart contract
// 6. Unpause mvx-esdt-safe smart contract
// 7. Deploy testing smart contract
// 8. Create a payments vector
// 9. Create transfer data that contains the banned endpoint name
// 10. Deposit
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_endpoint_banned_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from("hello")]),
        ManagedVec::new(),
    );

    chain_interactor
        .deploy_contracts(
            SovereignConfig::default_config(),
            OptionalValue::Some(config),
            None,
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some(BANNED_ENDPOINT_NAME),
            None,
        )
        .await;

    let address_states = vec![
        chain_interactor.state.current_fee_market_address().clone(),
        chain_interactor.state.current_testing_sc_address().clone(),
    ];

    chain_interactor
        .reset_state_chain_sim(Some(address_states))
        .await;
}

// NOTE: Add checks for account storage after finding out how to encode values in state

// Test that deposit works when there is transfer data and fee is enabled
// Steps:
// 1. Create the FeeStruct
// 2. Deploy header verifier smart contract
// 3. Deploy mvx-esdt-safe smart contract
// 4. Deploy fee-market smart contract
// 5. Set fee-market address in mvx-esdt-safe smart contract
// 6. Unpause mvx-esdt-safe smart contract
// 7. Deploy testing smart contract
// 8. Create a payments vector containing the fee payment as well
// 9. Create transfer data
// 10. Deposit
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_fee_enabled() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
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
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    chain_interactor
        .deploy_contracts(
            SovereignConfig::default_config(),
            OptionalValue::Some(config),
            Some(fee),
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let fee_amount = BigUint::from(10_000u64);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 1000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec,
            None,
            Some("deposit"),
        )
        .await;

    let address_states = vec![
        chain_interactor.state.current_fee_market_address().clone(),
        chain_interactor.state.current_testing_sc_address().clone(),
    ];

    chain_interactor
        .reset_state_chain_sim(Some(address_states))
        .await;
}

// Test that deposit works when there is only transfer data and fee is disabled
// Steps:
// 1. Deploy header verifier smart contract
// 2. Deploy mvx-esdt-safe smart contract
// 3. Deploy fee-market smart contract
// 4. Set fee-market address in mvx-esdt-safe smart contract
// 5. Unpause mvx-esdt-safe smart contract
// 6. Deploy testing smart contract
// 7. Create transfer data
// 8. Deposit and check logs
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_only_transfer_data_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    chain_interactor
        .deploy_contracts(
            SovereignConfig::default_config(),
            OptionalValue::Some(config),
            None,
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let gas_limit = 1000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::Some(transfer_data),
            ManagedVec::new(),
            None,
            Some("scCall"),
        )
        .await;

    let address_states = vec![
        chain_interactor.state.current_fee_market_address().clone(),
        chain_interactor.state.current_testing_sc_address().clone(),
    ];

    chain_interactor
        .reset_state_chain_sim(Some(address_states))
        .await;
}

// Test that deposit fails when payment does not cover fee
// Steps:
// 1. Create the FeeStruct
// 2. Deploy header verifier smart contract
// 3. Deploy mvx-esdt-safe smart contract
// 4. Deploy fee-market smart contract
// 5. Set fee-market address in mvx-esdt-safe smart contract
// 6. Unpause mvx-esdt-safe smart contract
// 7. Deploy testing smart contract
// 8. Create a payments vector without the fee payment
// 9. Create transfer data
// 10. Deposit
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_payment_does_not_cover_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
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
        base_token: TokenIdentifier::from(FIRST_TEST_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FIRST_TEST_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    chain_interactor
        .deploy_contracts(
            SovereignConfig::default_config(),
            OptionalValue::Some(config),
            Some(fee),
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(1u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 10_000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some(PAYMENT_DOES_NOT_COVER_FEE),
            None,
        )
        .await;

    let address_states = vec![
        chain_interactor.state.current_fee_market_address().clone(),
        chain_interactor.state.current_testing_sc_address().clone(),
    ];

    chain_interactor
        .reset_state_chain_sim(Some(address_states))
        .await;
}

// TODO: add deposit_refund_fee test after finding a method to check for balance

// Test that register token fails when the token type is invalid
// Steps:
// 1. Deploy header verifier smart contract
// 2. Deploy mvx-esdt-safe smart contract
// 3. Create the token properties
// 4. Register the token
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_invalid_type_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_chain_config(SovereignConfig::default_config())
        .await;

    chain_interactor
        .deploy_header_verifier(
            chain_interactor
                .state
                .current_chain_config_sc_address()
                .clone(),
        )
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
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

    chain_interactor.reset_state_chain_sim(None).await;
}

// Test that register token works when the token type is fungible
// Steps:
// 1. Deploy header verifier smart contract
// 2. Deploy mvx-esdt-safe smart contract
// 3. Create the token properties
// 4. Register the token
// 5. Check that the token ticker exists in sov-to-mvx-token storage
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_fungible_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_chain_config(SovereignConfig::default_config())
        .await;

    chain_interactor
        .deploy_header_verifier(
            chain_interactor
                .state
                .current_chain_config_sc_address()
                .clone(),
        )
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
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

    chain_interactor.reset_state_chain_sim(None).await;
}

// Test that register token works when the token type is non-fungible
// Steps:
// 1. Deploy header verifier smart contract
// 2. Deploy mvx-esdt-safe smart contract
// 3. Create the token properties
// 4. Register the token
// 5. Check that the token ticker exists in sov-to-mvx-token storage
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_non_fungible_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_chain_config(SovereignConfig::default_config())
        .await;

    chain_interactor
        .deploy_header_verifier(
            chain_interactor
                .state
                .current_chain_config_sc_address()
                .clone(),
        )
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
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

    chain_interactor.reset_state_chain_sim(None).await;
}

// Test that register token works when the token type is dynamic non-fungible
// Steps:
// 1. Deploy header verifier smart contract
// 2. Deploy mvx-esdt-safe smart contract
// 3. Create the token properties
// 4. Register the token
// 5. Check that the token ticker exists in sov-to-mvx-token storage
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_dynamic_non_fungible_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_chain_config(SovereignConfig::default_config())
        .await;

    chain_interactor
        .deploy_header_verifier(
            chain_interactor
                .state
                .current_chain_config_sc_address()
                .clone(),
        )
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
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

    chain_interactor.reset_state_chain_sim(None).await;
}

// Test that execute operation fails when the esdt safe address is not registered
// Steps:
// 1. Deploy header verifier smart contract
// 2. Deploy mvx-esdt-safe smart contract
// 3. Deploy fee-market smart contract
// 4. Set fee-market address in mvx-esdt-safe smart contract
// 5. Unpause mvx-esdt-safe smart contract
// 6. Create a payment for the operation
// 7. Create operation and hash of hashes
// 8. Execute operation
// 9. Check that operation-hash-status storage is empty
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn execute_operation_no_esdt_safe_registered() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_chain_config(SovereignConfig::default_config())
        .await;

    chain_interactor
        .deploy_header_verifier(
            chain_interactor
                .state
                .current_chain_config_sc_address()
                .clone(),
        )
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    chain_interactor.unpause_endpoint().await;

    chain_interactor.deploy_testing_sc().await;

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        EsdtTokenData::default(),
    );

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.owner_address),
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
        .execute_operations(
            hash_of_hashes,
            operation.clone(),
            Some(SETUP_PHASE_NOT_COMPLETED),
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

    let address_states = vec![
        chain_interactor.state.current_testing_sc_address().clone(),
        chain_interactor.state.current_fee_market_address().clone(),
    ];

    chain_interactor
        .reset_state_chain_sim(Some(address_states))
        .await;
}

// Test that execute operation works in the happy flow
// Steps:
// 1. Create payment vector
// 2. Create transfer data
// 3. Deploy header verifier smart contract
// 4. Deploy mvx-esdt-safe smart contract
// 5. Deploy fee-market smart contract
// 6. Set fee-market address in mvx-esdt-safe smart contract
// 7. Unpause mvx-esdt-safe smart contract
// 8. Deploy testing smart contract
// 9. Create operation
// 10. Deposit and check logs
// 11. Set mvx-esdt-safe address in header verifier smart contract
// 12. Create operation hashes
// 13. Deploy chain config smart contract
// 14. Register operation
// 15. Check that operation-hash-status storage has value OperationHashStatus::NotLocked
// 16. Execute operation and check logs
// 17. Check that operation-hash-status storage is empty
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn execute_operation_success_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let token_data = EsdtTokenData {
        amount: BigUint::from(10_000_000_000_000_000_000u128), // 10 Tokens
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(TokenIdentifier::from(FIRST_TEST_TOKEN), 0, token_data);
    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from_esdt_bytes(FIRST_TEST_TOKEN.as_str()),
        token_nonce: 0,
        amount: BigUint::from(10_000_000_000_000_000_000u128),
    });

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.owner_address),
        Some(transfer_data),
    );

    chain_interactor
        .deploy_contracts(
            SovereignConfig::default_config(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
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
        .deposit(
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

    chain_interactor
        .set_esdt_safe_address_in_header_verifier(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone(),
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
        .execute_operations(
            hash_of_hashes,
            operation.clone(),
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

    let address_states = vec![
        chain_interactor.state.current_fee_market_address().clone(),
        chain_interactor.state.current_testing_sc_address().clone(),
    ];

    chain_interactor
        .reset_state_chain_sim(Some(address_states))
        .await;
}

// Test that execute operation works when there is only transfer data and fee is disabled
// Steps:
// 1. Create transfer data
// 2. Deploy header verifier smart contract
// 3. Deploy mvx-esdt-safe smart contract
// 4. Deploy fee-market smart contract
// 5. Set fee-market address in mvx-esdt-safe smart contract
// 6. Unpause mvx-esdt-safe smart contract
// 7. Deploy testing smart contract
// 8. Create operation
// 9. Set mvx-esdt-safe address in header verifier smart contract
// 10. Create operation hashes
// 11. Deploy chain config smart contract
// 12. Register operation
// 13. Check that operation-hash-status storage has value OperationHashStatus::NotLocked
// 14. Execute operation and check logs
// 15. Check that operation-hash-status storage is empty
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn execute_operation_only_transfer_data_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.owner_address),
        Some(transfer_data),
    );

    chain_interactor
        .deploy_contracts(
            SovereignConfig::default_config(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
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

    chain_interactor
        .set_esdt_safe_address_in_header_verifier(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone(),
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
        .execute_operations(
            hash_of_hashes,
            operation.clone(),
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

    let address_states = vec![
        chain_interactor.state.current_fee_market_address().clone(),
        chain_interactor.state.current_testing_sc_address().clone(),
        chain_interactor
            .state
            .current_chain_config_sc_address()
            .clone(),
    ];

    chain_interactor
        .reset_state_chain_sim(Some(address_states))
        .await;
}
