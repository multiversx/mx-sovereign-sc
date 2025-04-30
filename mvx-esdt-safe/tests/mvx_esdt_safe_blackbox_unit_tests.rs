use common_test_setup::constants::{
    CHAIN_CONFIG_ADDRESS, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_TOKEN, FIRST_TEST_TOKEN,
    HEADER_VERIFIER_ADDRESS, ONE_HUNDRED_MILLION, ONE_HUNDRED_THOUSAND, OWNER_ADDRESS,
    SECOND_TEST_TOKEN, SOV_TOKEN, TESTING_SC_ADDRESS, USER,
};
use common_test_setup::RegisterTokenArgs;
use cross_chain::storage::CrossChainStorage;
use cross_chain::{DEFAULT_ISSUE_COST, MAX_GAS_PER_TRANSACTION};
use error_messages::{
    BANNED_ENDPOINT_NAME, CANNOT_REGISTER_TOKEN, ERR_EMPTY_PAYMENTS, GAS_LIMIT_TOO_HIGH,
    INVALID_TYPE, MAX_GAS_LIMIT_PER_TX_EXCEEDED, MINT_AND_BURN_ROLES_NOT_FOUND,
    NOTHING_TO_TRANSFER, NO_ESDT_SAFE_ADDRESS, PAYMENT_DOES_NOT_COVER_FEE,
    SETUP_PHASE_ALREADY_COMPLETED, TOKEN_ID_IS_NOT_TRUSTED, TOKEN_IS_FROM_SOVEREIGN,
    TOO_MANY_TOKENS,
};
use header_verifier::OperationHashStatus;
use multiversx_sc::imports::UserBuiltinProxy;
use multiversx_sc::types::MultiValueEncoded;
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EsdtTokenData, EsdtTokenPayment, EsdtTokenType, ManagedBuffer, ManagedVec,
        TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_scenario::ScenarioTxRun;
use multiversx_sc_scenario::{api::StaticApi, ScenarioTxWhitebox};
use mvx_esdt_safe::bridging_mechanism::{BridgingMechanism, TRUSTED_TOKEN_IDS};
use mvx_esdt_safe_blackbox_setup::MvxEsdtSafeTestState;
use proxies::fee_market_proxy::{FeeStruct, FeeType};
use setup_phase::SetupPhaseModule;
use structs::configs::SovereignConfig;
use structs::operation::TransferData;
use structs::{
    aliases::PaymentsVec,
    configs::EsdtSafeConfig,
    operation::{Operation, OperationData, OperationEsdtPayment},
};
mod mvx_esdt_safe_blackbox_setup;

#[test]
fn deploy() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
}

/// Test that deploy fails when the gas limit in the config is too high
#[test]
fn deploy_invalid_config() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        MAX_GAS_PER_TRANSACTION + 1,
        ManagedVec::new(),
    );

    state.update_configuration(config, Some(MAX_GAS_LIMIT_PER_TX_EXCEEDED));
}

/// This Test checks the flow for registering an invalid token
#[test]
fn register_token_invalid_type() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let sov_token_id = TestTokenIdentifier::new(FIRST_TEST_TOKEN);
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = FIRST_TEST_TOKEN;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(
        register_token_args,
        egld_payment,
        Some(CANNOT_REGISTER_TOKEN),
    );
}

/// This Test checks the flow for registering an invalid token with prefix
#[test]
fn register_token_invalid_type_with_prefix() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = TestTokenIdentifier::new(SOV_TOKEN);
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = FIRST_TEST_TOKEN;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, Some(INVALID_TYPE));

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(FIRST_TEST_TOKEN);
}

/// This Test checks the flow for registering a token that is not native
#[test]
fn register_token_not_native() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = TestTokenIdentifier::new(SECOND_TEST_TOKEN);
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = FIRST_TEST_TOKEN;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(
        register_token_args,
        egld_payment,
        Some(CANNOT_REGISTER_TOKEN),
    );
}

/// This Test checks the flow for registering a fungible token
#[test]
fn register_token_fungible_token() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = TestTokenIdentifier::new(SOV_TOKEN);
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = FIRST_TEST_TOKEN;
    let num_decimals = 3;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, None);
}

/// Test that register token works with a non-fungible token type
#[test]
fn register_token_nonfungible_token() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = TestTokenIdentifier::new(FIRST_TEST_TOKEN);
    let token_type = EsdtTokenType::NonFungible;
    let token_display_name = "TokenOne";
    let num_decimals = 0;
    let token_ticker = FIRST_TEST_TOKEN;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(
        register_token_args,
        egld_payment,
        Some(CANNOT_REGISTER_TOKEN),
    );
}

/// Test that deposit fails when there is no payment for transfer
#[test]
fn deposit_nothing_to_transfer() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);
    state.common_setup.deploy_fee_market(None);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::new(),
        Some(NOTHING_TO_TRANSFER),
        None,
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(FIRST_TEST_TOKEN);
}

/// Test that complete setup phase succeeds
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe SC
/// 2. Complete the setup phase
/// 3. Check the SCs storage after completing the setup phase
#[test]
fn complete_setup_phase() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(FIRST_TEST_TOKEN, token_display_name, egld_payment, None);
    state.complete_setup_phase(None, Some("unpauseContract"));

    state
        .common_setup
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc.is_setup_phase_complete());
        });
}

/// Test that complete setup phase fails when the setup phase was already completed
#[test]
fn complete_setup_phase_already_completed() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    state.complete_setup_phase(None, Some("unpauseContract"));
    state
        .common_setup
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc.is_setup_phase_complete());
        });

    state.complete_setup_phase(Some(SETUP_PHASE_ALREADY_COMPLETED), None);
}

/// Test that deposit fails when there are too many tokens in the payment (limit being the MAX_TRANSFERS_PER_TX)
#[test]
fn deposit_too_many_tokens() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);
    state.common_setup.deploy_fee_market(None);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);
    let esdt_token_payment = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::default(),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment; 11]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        payments_vec,
        Some(TOO_MANY_TOKENS),
        None,
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(FIRST_TEST_TOKEN);
}

/// Test that deposit with no transfer data succeeds
#[test]
fn deposit_no_transfer_data() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);
    state.common_setup.deploy_fee_market(None);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

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

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        payments_vec,
        None,
        Some("deposit"),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(FIRST_TEST_TOKEN);
}

/// Test that deposit fails when the gas limit is too high
#[test]
fn deposit_gas_limit_too_high() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(ManagedVec::new(), ManagedVec::new(), 1, ManagedVec::new());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.complete_setup_phase(None, None);
    state.common_setup.deploy_fee_market(None);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);
    state.common_setup.deploy_testing_sc();

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

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec,
        Some(GAS_LIMIT_TOO_HIGH),
        None,
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(FIRST_TEST_TOKEN);
}

/// Test that deposit fails when the endpoint is banned
#[test]
fn deposit_endpoint_banned() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from("hello")]),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.complete_setup_phase(None, None);
    state.common_setup.deploy_fee_market(None);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

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

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec,
        Some(BANNED_ENDPOINT_NAME),
        None,
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(FIRST_TEST_TOKEN);
}

// Test that deposit with no transfer data, no fee and no payment fails
#[test]
fn deposit_no_transfer_data_no_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);

    state.common_setup.deploy_fee_market(None);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::new(),
        Some(NOTHING_TO_TRANSFER),
        None,
    );
}

/// This test checks the flow for a deposit with transfer data only
/// Steps for this test:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 3. Deploy the Testing smart contract
/// 6. Create the ESDT token payments
/// 7. Create the payments vector
/// 8. Create the transfer data
/// 9. Call the deposit function
/// 10. Check the balances of the accounts
#[test]
fn deposit_transfer_data_only_no_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);

    state.common_setup.deploy_fee_market(None);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        PaymentsVec::new(),
        None,
        Some("scCall"),
    );
}

/// This test check the flow for a deposit with transfer data that fails
#[test]
fn deposit_transfer_data_only_with_fee_nothing_to_transfer() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        PaymentsVec::new(),
        Some(ERR_EMPTY_PAYMENTS),
        None,
    );
}

/// This test check the flow for a deposit with transfer data only and the fee is enabled
/// Steps for this test:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Deploy the Fee-Market smart contract
/// 3. Deploy the Testing smart contract
/// 4. Set the Fee-Market address
/// 5. Create the fee payment
/// 6. Create the ESDT token payments
/// 7. Create the payments vector
/// 8. Create the transfer data
/// 9. Call the deposit function
/// 10. Check the balances of the accounts
#[test]
fn deposit_transfer_data_only_with_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.complete_setup_phase(None, None);

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());
    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let payments_vec = PaymentsVec::from(fee_payment);

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec,
        None,
        Some("scCall"),
    );
}

/// This test check the flow for a deposit when the fee is enabled
/// Steps for this test:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Deploy the Fee-Market smart contract
/// 3. Deploy the Testing smart contract
/// 4. Set the Fee-Market address
/// 5. Create the fee payment
/// 6. Create the ESDT token payments
/// 7. Create the payments vector
/// 8. Create the transfer data
/// 9. Call the deposit function
/// 10. Check the balances of the accounts
#[test]
fn deposit_fee_enabled() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.complete_setup_phase(None, None);

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

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

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec.clone(),
        None,
        Some("deposit"),
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(FIRST_TEST_TOKEN),
            expected_amount_token_one,
        );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(SECOND_TEST_TOKEN),
            expected_amount_token_two,
        );

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(TokenIdentifier::from(FEE_TOKEN), expected_amount_token_fee);
}

/// Test that deposit fails when the payment does not cover the fee
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Deploy the Fee-Market smart contract
/// 3. Deploy the Testing smart contract
/// 4. Set the Fee-Market address
/// 5. Create the fee payment
/// 6. Create the ESDT token payments
/// 7. Create the payments vector
/// 8. Create the transfer data
/// 9. Call the deposit function
/// 10. Check the balances of the accounts
#[test]
fn deposit_payment_doesnt_cover_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.complete_setup_phase(None, None);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FIRST_TEST_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FIRST_TEST_TOKEN),
            per_transfer: BigUint::from(1u64),
            per_gas: BigUint::from(1u64),
        },
    };

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

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

    let gas_limit = 10_000;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec,
        Some(PAYMENT_DOES_NOT_COVER_FEE),
        None,
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(FIRST_TEST_TOKEN);
    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(SECOND_TEST_TOKEN);
}

/// Test that after deposit fails the tokens are refunded
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Deploy the Fee-Market smart contract
/// 3. Deploy the Testing smart contract
/// 4. Set the Fee-Market address
/// 5. Create the fee payment
/// 6. Create the ESDT token payments
/// 7. Create the payments vector
/// 8. Create the transfer data
/// 9. Call the deposit function
/// 10. Check the logs
/// 11. Check the balances of the accounts
#[test]
fn deposit_refund() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.complete_setup_phase(None, None);

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec.clone(),
        None,
        Some("deposit"),
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(FIRST_TEST_TOKEN),
            &expected_amount_token_one,
        );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(SECOND_TEST_TOKEN),
            &expected_amount_token_two,
        );

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(TokenIdentifier::from(FEE_TOKEN), expected_amount_token_fee);
}

/// Test that deposit with a burn mechanism works
#[test]
fn deposit_success_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles();
    state.complete_setup_phase(None, None);
    state.common_setup.deploy_fee_market(None);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    let esdt_token_payment_trusted_token = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TEST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        esdt_token_payment_trusted_token.clone(),
        esdt_token_payment_two.clone(),
    ]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        payments_vec,
        None,
        Some("deposit"),
    );

    state
        .common_setup
        .check_multiversx_to_sovereign_token_id_mapper_is_empty(TRUSTED_TOKEN_IDS[0]);

    state.common_setup.check_sc_esdt_balance(
        vec![
            MultiValue3::from((TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]), 0u64, 0u64)),
            MultiValue3::from((TestTokenIdentifier::new(SECOND_TEST_TOKEN), 100, 0)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
        mvx_esdt_safe::contract_obj,
    )
}

/// Test that register token works with a valid prefix
#[test]
fn register_token_fungible_token_with_prefix() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = TestTokenIdentifier::new(SOV_TOKEN);
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = FIRST_TEST_TOKEN;
    let num_decimals = 3;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, None);

    // TODO: Add check for storage after callback issue is fixed
}

/// Test that register token fails when token has no prefix
#[test]
fn register_token_fungible_token_no_prefix() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let sov_token_id = TestTokenIdentifier::new(FIRST_TEST_TOKEN);
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = FIRST_TEST_TOKEN;
    let num_decimals = 3;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id: sov_token_id.into(),
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(
        register_token_args,
        egld_payment,
        Some(CANNOT_REGISTER_TOKEN),
    );
}

/// Test that register token fails if the token is already registered
#[test]
fn register_native_token_already_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(
        FIRST_TEST_TOKEN,
        token_display_name,
        egld_payment.clone(),
        None,
    );

    // TODO: Add check for storage after callback issue is fixed

    state.register_native_token(
        FIRST_TEST_TOKEN,
        token_display_name,
        egld_payment.clone(),
        None,
        // NOTE: Some(NATIVE_TOKEN_ALREADY_REGISTERED) when fix is here,
    );
}

/// Test that register native works in the happy flow
#[test]
fn register_native_token() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(
        FIRST_TEST_TOKEN,
        token_display_name,
        egld_payment.clone(),
        None,
    );

    // TODO: Check storage
}

/// Test that execute operation fails when the Mvx-ESDT-Safe address is not set in Header-Verifier contract
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Create the operation
/// 3. Create the hash of hashes
/// 4. Deploy the Header-Verifier smart contract
/// 5. Call the execute operation function
/// 6. Check the operation hash status
#[test]
fn execute_operation_no_esdt_safe_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);
    state.complete_setup_phase(None, None);

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from(FIRST_TEST_TOKEN),
        0,
        EsdtTokenData::default(),
    );

    let operation_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let hash_of_hashes = state.get_operation_hash(&operation);

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);
    state.execute_operation(
        &hash_of_hashes,
        &operation,
        Some(NO_ESDT_SAFE_ADDRESS),
        None,
    );

    state
        .common_setup
        .check_operation_hash_status_is_empty(&hash_of_hashes);
}

// /// Test that execute operation works in the happy flow
// /// Steps:
// /// 1. Deploy the Mvx-ESDT-Safe smart contract
// /// 2. Create the operation
// /// 3. Create the hash of hashes
// /// 4. Deploy the Header-Verifier smart contract
// /// 5. Deploy the Testing smart contract
// /// 6. Set the Mvx-ESDT-Safe address in the Header-Verifier smart contract
// /// 7. Call the register operation function
// /// 8. Call the execute operation function
// /// 9. Check the operation hash status
// #[test]
// fn execute_operation_success() {
//     let mut state = MvxEsdtSafeTestState::new();
//     let config = OptionalValue::Some(EsdtSafeConfig::default_config());
//     state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);
//
//     let token_data = EsdtTokenData {
//         amount: BigUint::from(100u64),
//         ..Default::default()
//     };
//
//     let payment = OperationEsdtPayment::new(TokenIdentifier::from(FIRST_TEST_TOKEN), 0, token_data);
//
//     let gas_limit = 1;
//     let function = ManagedBuffer::<StaticApi>::from("hello");
//     let args =
//         ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);
//
//     let transfer_data = TransferData::new(gas_limit, function, args);
//
//     let operation_data =
//         OperationData::new(1, OWNER_ADDRESS.to_managed_address(), Some(transfer_data));
//
//     let operation = Operation::new(
//         TESTING_SC_ADDRESS.to_managed_address(),
//         vec![payment].into(),
//         operation_data,
//     );
//
//     let operation_hash = state.get_operation_hash(&operation);
//     let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
//
//     state.common_setup.deploy_header_verifier();
//     state.common_setup.deploy_testing_sc();
//     state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);
//
//     let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));
//
//     state
//         .common_setup
//         .deploy_chain_config(SovereignConfig::default_config());
//     state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);
//
//     state
//         .common_setup
//         .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);
//
//     state.execute_operation(&hash_of_hashes, &operation, None, Some("executedBridgeOp"));
//
//     state
//         .common_setup
//         .check_operation_hash_status_is_empty(&operation_hash);
// }

// /// Test execute operation with native token happy flow
// /// Steps:
// /// 1. Deploy the Mvx-ESDT-Safe smart contract
// /// 2. Register the native token
// /// 3. Create the operation
// /// 4. Create the hash of hashes
// /// 5. Deploy the Header-Verifier smart contract
// /// 6. Deploy the Testing smart contract
// /// 7. Set the Mvx-ESDT-Safe address in the Header-Verifier smart contract
// /// 8. Call the register operation function
// /// 9. Call the execute operation function
// /// 10. Check the operation hash status
// #[test]
// fn execute_operation_with_native_token_success() {
//     let mut state = MvxEsdtSafeTestState::new();
//     let config = EsdtSafeConfig::default_config();
//     state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
//
//     let token_display_name = "TokenOne";
//     let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);
//
//     state.register_native_token(FIRST_TEST_TOKEN, token_display_name, egld_payment, None);
//
//     let token_data = EsdtTokenData {
//         amount: BigUint::from(100u64),
//         ..Default::default()
//     };
//
//     let payment = OperationEsdtPayment::new(TokenIdentifier::from(FIRST_TEST_TOKEN), 0, token_data);
//
//     let gas_limit = 1;
//     let function = ManagedBuffer::<StaticApi>::from("hello");
//     let args =
//         ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);
//
//     let transfer_data = TransferData::new(gas_limit, function, args);
//
//     let operation_data =
//         OperationData::new(1, OWNER_ADDRESS.to_managed_address(), Some(transfer_data));
//
//     let operation = Operation::new(
//         TESTING_SC_ADDRESS.to_managed_address(),
//         vec![payment].into(),
//         operation_data,
//     );
//
//     let operation_hash = state.get_operation_hash(&operation);
//     let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
//
//     state.common_setup.deploy_header_verifier();
//     state.common_setup.deploy_testing_sc();
//     state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);
//
//     let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));
//
//     state
//         .common_setup
//         .deploy_chain_config(SovereignConfig::default_config());
//     state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);
//
//     state
//         .common_setup
//         .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);
//
//     state.execute_operation(&hash_of_hashes, &operation, None, Some("executedBridgeOp"));
//
//     state
//         .common_setup
//         .check_operation_hash_status_is_empty(&operation_hash);
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
//             0u64,
//             0u64,
//         ))],
//         TESTING_SC_ADDRESS.to_managed_address(),
//         testing_sc::contract_obj,
//     );
// }

// /// This test checks the succsesful flow of executing an `operation` with burn mechanism
// /// Steps for this test:
// /// 1. Deploy the Mvx-ESDT-Safe SC with roles for the trusted token
// /// 2. Create the `operation`
// /// 3. Deploy the needed smart contract (Header-Verifier, Fee-Market with no fee and Testing SC)
// /// 4. Set the Fee-Market address in Header-Verifier
// /// 5. Register the `operation`
// /// 6. Register the native token
// /// 7. Set the bridging mechanism to burn&mint
// /// 8. Execute the `operation`
// /// 9. Check if the registered `operation` hash status is empty
// /// 10. Check the balances for the owner, Mvx-ESDT-Safe and Testing SC
// #[test]
// fn execute_operation_burn_mechanism_without_deposit_cannot_subtract() {
//     let mut state = MvxEsdtSafeTestState::new();
//     state.deploy_contract_with_roles();
//
//     let token_data = EsdtTokenData {
//         amount: BigUint::from(100u64),
//         ..Default::default()
//     };
//
//     let payment =
//         OperationEsdtPayment::new(TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]), 0, token_data);
//
//     let operation_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);
//
//     let operation = Operation::new(
//         TESTING_SC_ADDRESS.to_managed_address(),
//         vec![payment].into(),
//         operation_data,
//     );
//
//     let operation_hash = state.get_operation_hash(&operation);
//     let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
//
//     state.common_setup.deploy_header_verifier();
//     state.common_setup.deploy_testing_sc();
//     state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);
//
//     let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));
//
//     state
//         .common_setup
//         .deploy_chain_config(SovereignConfig::default_config());
//
//     state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);
//
//     let token_display_name = "NativeToken";
//     let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);
//
//     state.register_native_token(TRUSTED_TOKEN_IDS[0], token_display_name, egld_payment, None);
//     state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);
//
//     state.execute_operation(&hash_of_hashes, &operation, None, Some("executedBridgeOp"));
//
//     state
//         .common_setup
//         .check_operation_hash_status_is_empty(&operation_hash);
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
//             0u64,
//             0u64,
//         ))],
//         ESDT_SAFE_ADDRESS.to_managed_address(),
//         mvx_esdt_safe::contract_obj,
//     );
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
//             0u64,
//             0u64,
//         ))],
//         TESTING_SC_ADDRESS.to_managed_address(),
//         testing_sc::contract_obj,
//     );
// }

// /// This test checks the succsesful flow of executing an `operation` with burn mechanism
// /// Steps for this test:
// /// 1. Deploy the Mvx-ESDT-Safe SC with roles for the trusted token
// /// 2. Create the `operation`
// /// 3. Deploy the needed smart contract (Header-Verifier, Fee-Market with no fee and Testing SC)
// /// 4. Set the Fee-Market address in Mvx-ESDT-Safe and Header-Verifier
// /// 5. Deposit the `payment`
// /// 6. Check for the deposit log
// /// 7. Register the `operation`
// /// 8. Check if the registered `operation` is not locked
// /// 9. Set the briding mechanism to burn&mint
// /// 10. Execute the `operation`
// /// 11. Check the balances for the owner, Mvx-ESDT-Safe and Testing SC
// /// 12. Check if the `operation` hash was removed from the Header-Verifier SC
// #[test]
// fn execute_operation_success_burn_mechanism() {
//     let mut state = MvxEsdtSafeTestState::new();
//     state.deploy_contract_with_roles();
//
//     let token_data = EsdtTokenData {
//         amount: BigUint::from(100u64),
//         ..Default::default()
//     };
//
//     let payment = OperationEsdtPayment::new(
//         TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]),
//         0,
//         token_data.clone(),
//     );
//
//     let operation_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);
//
//     let operation = Operation::new(
//         TESTING_SC_ADDRESS.to_managed_address(),
//         vec![payment.clone()].into(),
//         operation_data,
//     );
//
//     let operation_hash = state.get_operation_hash(&operation);
//     let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
//
//     state.common_setup.deploy_header_verifier();
//     state.common_setup.deploy_testing_sc();
//     state.common_setup.deploy_fee_market(None);
//     state.set_fee_market_address(FEE_MARKET_ADDRESS);
//     state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);
//
//     let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));
//
//     state.deposit(
//         USER.to_managed_address(),
//         OptionalValue::None,
//         PaymentsVec::from(vec![payment]),
//         None,
//         Some("deposit"),
//     );
//
//     state
//         .common_setup
//         .deploy_chain_config(SovereignConfig::default_config());
//
//     state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);
//
//     state
//         .common_setup
//         .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);
//
//     state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);
//
//     state.execute_operation(&hash_of_hashes, &operation, None, Some("executedBridgeOp"));
//
//     let expected_amount_trusted_token = BigUint::from(ONE_HUNDRED_MILLION) - &token_data.amount;
//
//     state
//         .common_setup
//         .world
//         .check_account(OWNER_ADDRESS)
//         .esdt_balance(
//             TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]),
//             &expected_amount_trusted_token,
//         );
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
//             0u64,
//             0u64,
//         ))],
//         ESDT_SAFE_ADDRESS.to_managed_address(),
//         mvx_esdt_safe::contract_obj,
//     );
//
//     state
//         .common_setup
//         .check_deposited_tokens_amount(vec![(TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]), 0)]);
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
//             0u64,
//             100u64,
//         ))],
//         TESTING_SC_ADDRESS.to_managed_address(),
//         testing_sc::contract_obj,
//     );
//
//     state
//         .common_setup
//         .check_operation_hash_status_is_empty(&operation_hash);
// }

// /// This test checks the flow of multiple deposit and executes along side bridging mechanism
// /// Steps for this test:
// /// 1. Deploy the Mvx-ESDT-Safe SC with roles for the trusted token
// /// 2. Deploy the needed smart contract (Header-Verifier, Fee-Market with no fee and Testing SC)
// /// 3. Set the Fee-Market address in Mvx-ESDT-Safe and Header-Verifier
// /// 4. Deposit the `deposit_payment` to the `USER`
// /// 5. Check for logs and esdt balance
// /// 6. Switch the bridging mechanism to Burn&Mint for the trusted token
// /// 7. Check for `deposited_tokens_amount` mapper and esdt balance
// /// 8. Create the first `operation`
// /// 9. Register the `operation`
// /// 10. Execute the `operation`
// /// 11. Check for `deposited_tokens_amount` mapper and esdt balance
// /// 12. Second deposit of `deposit_payment` to the `USER`
// /// 13. Check for logs, `deposited_tokens_amount` mapper and esdt balance
// /// 14. Set bridging mechanism back to Lock&Send
// /// 15. Check `deposited_tokens_amount` mapper and esdt balance
// /// 16. Create the second `operation`
// /// 17. Register the `operation`
// /// 18. Execute the `operation`
// /// 19. Check for `deposited_tokens_amount` mapper and esdt balance
// /// 12. Third deposit of `deposit_payment` to the `USER`
// /// 19. Check for logs, `deposited_tokens_amount` mapper and esdt balance
// #[test]
// fn deposit_execute_switch_mechanism() {
//     let mut state = MvxEsdtSafeTestState::new();
//     state.deploy_contract_with_roles();
//
//     let trusted_token_id = TRUSTED_TOKEN_IDS[0];
//
//     state.common_setup.deploy_header_verifier();
//     state.common_setup.deploy_testing_sc();
//     state.common_setup.deploy_fee_market(None);
//     state.set_fee_market_address(FEE_MARKET_ADDRESS);
//     state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);
//
//     let deposited_trusted_token_payment_amount = 1000u64;
//     let deposit_trusted_token_payment_token_data = EsdtTokenData {
//         amount: BigUint::from(deposited_trusted_token_payment_amount),
//         ..Default::default()
//     };
//     let deposit_trusted_token_payment = OperationEsdtPayment::new(
//         TokenIdentifier::from(trusted_token_id),
//         0,
//         deposit_trusted_token_payment_token_data,
//     );
//
//     state.deposit(
//         USER.to_managed_address(),
//         OptionalValue::None,
//         PaymentsVec::from(vec![deposit_trusted_token_payment.clone()]),
//         None,
//         Some("deposit"),
//     );
//
//     state
//         .common_setup
//         .world
//         .check_account(ESDT_SAFE_ADDRESS)
//         .esdt_balance(TestTokenIdentifier::new(trusted_token_id), 1000);
//
//     state.set_token_burn_mechanism(trusted_token_id, None);
//
//     let mut expected_deposited_amount = deposited_trusted_token_payment_amount;
//
//     state.common_setup.check_deposited_tokens_amount(vec![(
//         TestTokenIdentifier::new(trusted_token_id),
//         expected_deposited_amount,
//     )]);
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(trusted_token_id),
//             0,
//             0u64,
//         ))],
//         ESDT_SAFE_ADDRESS.to_managed_address(),
//         mvx_esdt_safe::contract_obj,
//     );
//
//     let execute_trusted_token_payment_amount = 500u64;
//     let execute_trusted_token_payment_token_data = EsdtTokenData {
//         amount: BigUint::from(execute_trusted_token_payment_amount),
//         ..Default::default()
//     };
//     let execute_trusted_token_payment = OperationEsdtPayment::new(
//         TokenIdentifier::from(trusted_token_id),
//         0,
//         execute_trusted_token_payment_token_data,
//     );
//     let operation_one_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);
//     let operation_one = Operation::new(
//         TESTING_SC_ADDRESS.to_managed_address(),
//         vec![execute_trusted_token_payment.clone()].into(),
//         operation_one_data,
//     );
//     let operation_one_hash = state.get_operation_hash(&operation_one);
//     let hash_of_hashes_one = ManagedBuffer::new_from_bytes(&sha256(&operation_one_hash.to_vec()));
//     let operations_hashes_one =
//         MultiValueEncoded::from(ManagedVec::from(vec![operation_one_hash.clone()]));
//
//     state.register_operation(
//         ManagedBuffer::new(),
//         &hash_of_hashes_one,
//         operations_hashes_one,
//     );
//
//     state.execute_operation(
//         &hash_of_hashes_one,
//         &operation_one,
//         None,
//         Some("executedBridgeOp"),
//     );
//
//     let mut expected_receiver_amount = execute_trusted_token_payment_amount;
//     expected_deposited_amount -= execute_trusted_token_payment_amount;
//
//     state.common_setup.check_deposited_tokens_amount(vec![(
//         TestTokenIdentifier::new(trusted_token_id),
//         expected_deposited_amount,
//     )]);
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(trusted_token_id),
//             0,
//             0u64,
//         ))],
//         ESDT_SAFE_ADDRESS.to_managed_address(),
//         mvx_esdt_safe::contract_obj,
//     );
//
//     state.deposit(
//         USER.to_managed_address(),
//         OptionalValue::None,
//         PaymentsVec::from(vec![deposit_trusted_token_payment.clone()]),
//         None,
//         Some("deposit"),
//     );
//
//     expected_deposited_amount += deposited_trusted_token_payment_amount;
//
//     state.common_setup.check_deposited_tokens_amount(vec![(
//         TestTokenIdentifier::new(trusted_token_id),
//         expected_deposited_amount,
//     )]);
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(trusted_token_id),
//             0,
//             0u64,
//         ))],
//         ESDT_SAFE_ADDRESS.to_managed_address(),
//         mvx_esdt_safe::contract_obj,
//     );
//
//     state.set_token_lock_mechanism(trusted_token_id, None);
//
//     state
//         .common_setup
//         .check_deposited_tokens_amount(vec![(TestTokenIdentifier::new(trusted_token_id), 0)]);
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(trusted_token_id),
//             0,
//             expected_deposited_amount,
//         ))],
//         ESDT_SAFE_ADDRESS.to_managed_address(),
//         mvx_esdt_safe::contract_obj,
//     );
//
//     let operation_two_data = OperationData::new(2, OWNER_ADDRESS.to_managed_address(), None);
//     let operation_two = Operation::new(
//         TESTING_SC_ADDRESS.to_managed_address(),
//         vec![execute_trusted_token_payment.clone()].into(),
//         operation_two_data,
//     );
//     let operation_two_hash = state.get_operation_hash(&operation_two);
//     let hash_of_hashes_two = ManagedBuffer::new_from_bytes(&sha256(&operation_two_hash.to_vec()));
//     let operations_hashes_two =
//         MultiValueEncoded::from(ManagedVec::from(vec![operation_two_hash.clone()]));
//
//     state.register_operation(
//         ManagedBuffer::new(),
//         &hash_of_hashes_two,
//         operations_hashes_two,
//     );
//
//     state.execute_operation(
//         &hash_of_hashes_two,
//         &operation_two,
//         None,
//         Some("executedBridgeOp"),
//     );
//
//     state
//         .common_setup
//         .check_deposited_tokens_amount(vec![(TestTokenIdentifier::new(trusted_token_id), 0)]);
//
//     expected_receiver_amount += execute_trusted_token_payment_amount;
//     expected_deposited_amount -= execute_trusted_token_payment_amount;
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(trusted_token_id),
//             0,
//             expected_deposited_amount,
//         ))],
//         ESDT_SAFE_ADDRESS.to_managed_address(),
//         mvx_esdt_safe::contract_obj,
//     );
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(trusted_token_id),
//             0,
//             expected_receiver_amount,
//         ))],
//         TESTING_SC_ADDRESS.to_managed_address(),
//         testing_sc::contract_obj,
//     );
//
//     state.deposit(
//         USER.to_managed_address(),
//         OptionalValue::None,
//         PaymentsVec::from(vec![deposit_trusted_token_payment]),
//         None,
//         Some("deposit"),
//     );
//
//     expected_deposited_amount += deposited_trusted_token_payment_amount;
//
//     state
//         .common_setup
//         .check_deposited_tokens_amount(vec![(TestTokenIdentifier::new(trusted_token_id), 0)]);
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(trusted_token_id),
//             0,
//             expected_deposited_amount,
//         ))],
//         ESDT_SAFE_ADDRESS.to_managed_address(),
//         testing_sc::contract_obj,
//     );
//
//     state.common_setup.check_sc_esdt_balance(
//         vec![MultiValue3::from((
//             TestTokenIdentifier::new(trusted_token_id),
//             0,
//             expected_receiver_amount,
//         ))],
//         TESTING_SC_ADDRESS.to_managed_address(),
//         testing_sc::contract_obj,
//     );
// }

// /// This test checks the flow of executing an Operation with no payments
// /// Steps for this test:
// /// 1. Deploy the Mvx-ESDT-Safe SC with the default config
// /// 2. Registed the native token
// /// 3. Create the `operation`
// /// 4. Deploy the needed smart contract (Header-Verifier, Fee-Market with no fee and Testing SC)
// /// 5. Register the `operation`
// /// 6. Check if the registered `operation` is not locked
// /// 7. Execute the `operation`
// /// 8. Check the emited logs
// /// 9. Check if the `operation` hash was removed from the Header-Verifier SC
// #[test]
// fn execute_operation_no_payments() {
//     let mut state = MvxEsdtSafeTestState::new();
//     state.deploy_contract(
//         HEADER_VERIFIER_ADDRESS,
//         OptionalValue::Some(EsdtSafeConfig::default_config()),
//     );
//
//     let token_display_name = "TokenOne";
//     let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);
//
//     state.register_native_token(FIRST_TEST_TOKEN, token_display_name, egld_payment, None);
//
//     let gas_limit = 1;
//     let function = ManagedBuffer::<StaticApi>::from("hello");
//     let args =
//         ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);
//
//     let transfer_data = TransferData::new(gas_limit, function, args);
//
//     let operation_data =
//         OperationData::new(1, OWNER_ADDRESS.to_managed_address(), Some(transfer_data));
//
//     let operation = Operation::new(
//         TESTING_SC_ADDRESS.to_managed_address(),
//         ManagedVec::new(),
//         operation_data,
//     );
//
//     let operation_hash = state.get_operation_hash(&operation);
//     let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
//
//     state.common_setup.deploy_header_verifier();
//     state.common_setup.deploy_testing_sc();
//     state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);
//
//     let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));
//
//     state
//         .common_setup
//         .deploy_chain_config(SovereignConfig::default_config());
//
//     state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);
//
//     state
//         .common_setup
//         .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);
//
//     state.execute_operation(&hash_of_hashes, &operation, None, Some("executedBridgeOp"));
//
//     state
//         .common_setup
//         .check_operation_hash_status_is_empty(&operation_hash);
// }

/// This test checks the flow of executing an Operation with no payments
/// which should emit a failed event
/// Steps for this test:
/// 1. Deploy the Mvx-ESDT-Safe SC with the default config
/// 2. Registed the native token
/// 3. Create the `operation`
/// 4. Deploy the needed smart contract (Header-Verifier, Fee-Market with no fee and Testing SC)
/// 5. Register the `operation`
/// 6. Check if the registered `operation` is not locked
/// 7. Execute the `operation`
/// 8. Check the emited logs
/// 9. Check if the `operation` hash was removed from the Header-Verifier SC
#[test]
fn execute_operation_no_payments_failed_event() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());

    state
        .common_setup
        .deploy_header_verifier(&CHAIN_CONFIG_ADDRESS);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let token_display_name = "TokenOne";
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    state.register_native_token(FIRST_TEST_TOKEN, token_display_name, egld_payment, None);

    state.complete_setup_phase(None, None);

    // state
    //     .common_setup
    //     .world
    //     .tx()
    //     .from(HEADER_VERIFIER_ADDRESS)
    //     .to(ESDT_SAFE_ADDRESS)
    //     .typed(UserBuiltinProxy)
    //     .change_owner_address(&OWNER_ADDRESS.to_managed_address())
    //     .run();

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("WRONG_ENDPOINT");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data =
        OperationData::new(1, OWNER_ADDRESS.to_managed_address(), Some(transfer_data));

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        ManagedVec::new(),
        operation_data,
    );

    let operation_hash = state.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state.common_setup.deploy_testing_sc();
    state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);

    state
        .common_setup
        .check_operation_hash_status(&operation_hash, OperationHashStatus::NotLocked);

    state.execute_operation(&hash_of_hashes, &operation, None, Some("executedBridgeOp"));

    state
        .common_setup
        .check_operation_hash_status_is_empty(&operation_hash);
}

/// This Test checks the flow for setting the token burn mechanism without having roles
#[test]
fn set_token_burn_mechanism_no_roles() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    state.set_token_burn_mechanism("WEGLD", Some(MINT_AND_BURN_ROLES_NOT_FOUND));
}

/// This Test checks the flow setting the bridging mechanism for a untrusted token
#[test]
fn set_token_burn_mechanism_token_not_trusted() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(FIRST_TEST_TOKEN, Some(TOKEN_ID_IS_NOT_TRUSTED));
}

/// This Test checks the flow setting the bridging mechanism to burn&mint
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Set token burn mechanism for any trusted token
/// 3. Check sc storage and balance
#[test]
fn set_token_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state
        .common_setup
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc
                .burn_mechanism_tokens()
                .contains(&TokenIdentifier::from(TRUSTED_TOKEN_IDS[0])))
        });

    state.common_setup.check_sc_esdt_balance(
        vec![MultiValue3::from((
            TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
            0,
            0,
        ))],
        ESDT_SAFE_ADDRESS.to_managed_address(),
        mvx_esdt_safe::contract_obj,
    );
}

/// This Test checks the flow setting the bridging mechanism to lock&send
/// Steps:
/// 1. Deploy the Mvx-ESDT-Safe smart contract
/// 2. Set token burn mechanism for any trusted token
/// 3. Set token lock mech
/// 3. Check sc storage and balance
#[test]
fn set_token_lock_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);
    state.set_token_lock_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state
        .common_setup
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc.burn_mechanism_tokens().is_empty())
        });

    state.common_setup.check_sc_esdt_balance(
        vec![MultiValue3::from((
            TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
            100,
            0,
        ))],
        ESDT_SAFE_ADDRESS.to_managed_address(),
        mvx_esdt_safe::contract_obj,
    );
}

/// This Test checks the flow setting the bridging mechanism to burn&mint of a Sovereign token
#[test]
fn set_token_lock_mechanism_token_from_sovereign() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            sc.multiversx_to_sovereign_token_id_mapper(&TokenIdentifier::from(
                TRUSTED_TOKEN_IDS[0],
            ))
            .set(TokenIdentifier::from("MOCK"));
        });

    state.set_token_lock_mechanism(TRUSTED_TOKEN_IDS[0], Some(TOKEN_IS_FROM_SOVEREIGN));
}
